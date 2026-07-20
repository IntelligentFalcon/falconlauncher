#![allow(deprecated)]

use crate::models::versions::MinecraftVersion;
use crate::services::directory_manager::{
    get_assets_directory, get_falcon_launcher_directory, get_libraries_directory,
    get_minecraft_directory, get_natives_folder, get_temp_directory, get_version_directory,
    get_versions_directory,
};
use crate::services::utils::{
    update_download, update_download_bar, update_download_status,
};
use crate::services::utils::{convert_to_full_path, convert_to_full_url, verify_file_existence};
use crate::services::version_manager::load_version_manifest;

use crate::models::config::Config;
use crate::models::downloader::{
    library_from_value, AssetIndex, AssetObjects, DownloadDetail, ForgeInstallProfile,
    ForgeVersionJsonInfo, Library, LibraryArtifact, LibraryRules, Manifest,
    MinecraftManifestVersion, Rule, VersionLoader,
};
use crate::models::error::{
    download_error, io_err_permission, io_err_read_file, json_read_err, launcher_file_not_found,
    launcher_manifest_not_found, request_unknown_err, Returns, Void,
};
use crate::models::fabric::{FabricInstaller, FabricLoader, FabricMinecraftVersion};
use crate::models::logger::{info_launcher, LogLine};
use crate::models::mirror::{mirror_from, Mirror};
use crate::models::platform::get_current_os;
use crate::services::jdk_manager::{download_java, get_java};
use std::collections::HashMap;
use std::fs;
use std::fs::{create_dir_all, exists, set_permissions, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use sha1::{Digest, Sha1};
use tauri::async_runtime::block_on;
use tauri::AppHandle;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::UnboundedSender;
use zip::ZipArchive;
use zip_extract::extract;
use crate::GLOBAL_CACHE;

pub async fn download_version(
    version: &MinecraftVersion,
    app_handle: &AppHandle,
    logger: &UnboundedSender<LogLine>,
    cfg: &Config,
) -> Void {
    let id = &version.id;
    let mirror = mirror_from(&cfg.download_settings.mirror);
    let manifest = load_version_manifest(&mirror).await;
    match manifest {
        None => {
            return Err(launcher_manifest_not_found());
        }
        Some(val) => {
            let res = download_from_manifest(id, &val, &mirror).await;
            if res.is_err() {
                return Err(launcher_file_not_found(format!("{id}.json")));
            }
        }
    }
    let content =
        fs::read_to_string(PathBuf::from(version.get_json())).map_err(|x| io_err_read_file(x))?;
    let json: MinecraftManifestVersion =
        serde_json::from_str(&content).map_err(|x| json_read_err(x))?;
    let java_version = json.java_version.unwrap();
    download_java(
        &java_version.component.to_string(),
        &java_version.major_version.to_string(),
        logger,
        &mirror,
    )
    .await?;

    download_libraries(&json.libraries, &id, app_handle, logger, &mirror).await?;
    if let Some(downloads) = &json.downloads {
        if let Some(client_download) = downloads.get("client") {
            logger.send(info_launcher("downloading client".to_string()));
            update_download_status("Downloading version...", &app_handle);
            download_client(client_download, &id, logger, &mirror).await?;
        }
    }
    if let Some(asset_index) = &json.asset_index {
        logger.send(info_launcher("downloading assets".to_string()));
        update_download_status("Downloading assets...", &app_handle);
        download_assets(asset_index, logger, &mirror, app_handle).await?;
    }
    if let Some(logging) = &json.logging {
        logger.send(info_launcher("downloading logging files".to_string()));
        download_file_if_not_exists(
            &get_version_directory(id).join(&logging.client.file.url.split("/").last().unwrap()),
            logging.client.file.url.clone(),
            logging.client.file.size,
        )
        .await?;
    }
    update_download(100, "Done", app_handle);
    Ok(())
}

async fn download_assets(
    value: &AssetIndex,
    logger: &UnboundedSender<LogLine>,
    mirror: &Mirror,
    app_handle: &AppHandle,
) -> Void {
    let id = &value.id;
    let url = mirror.parse_url(&value.url);
    let total_size = value.total_size;
    let asset_index_path = get_assets_directory()
        .join("indexes")
        .join(format!("{id}.json"))
        .to_str()
        .unwrap()
        .to_string();
    download_file_if_not_exists(
        &PathBuf::from(&asset_index_path),
        url.to_string(),
        total_size,
    )
    .await?;
    let content =
        fs::read_to_string(PathBuf::from(&asset_index_path)).expect("Failed to read file.");

    let json: AssetObjects =
        serde_json::from_str(content.as_str()).expect("JSON File isn't well formatted.");
    let url_template = "https://resources.download.minecraft.net/{id}/{hash}";

    let total_objects = json.objects.len();
    for (i, (_name, asset_entry)) in json.objects.iter().enumerate() {
        let hash = &asset_entry.hash;
        let prefix_id = hash[0..2].to_string();
        let size = asset_entry.size;
        let url = mirror.parse_url(
            &url_template
                .replace("{id}", prefix_id.as_str())
                .replace("{hash}", hash),
        );
        let path = get_assets_directory()
            .join("objects")
            .join(prefix_id.as_str())
            .join(hash);
        update_download_bar((i * 100 / total_objects) as i64, app_handle);
        download_file_if_not_exists(&path, url, size).await?;
    }
    Ok(())
}


/// Verifies if a file matches the expected SHA-1 hash.
/// The `expected_sha1` parameter can be either uppercase or lowercase.
pub async fn verify_file_hash<P: AsRef<Path>>(path: P, expected_sha1: &str) -> tokio::io::Result<bool> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 8192]; // 8KB chunks

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    let calculated_sha1 = hex::encode(result);

    // Case-insensitive comparison
    Ok(calculated_sha1.eq_ignore_ascii_case(expected_sha1))
}
pub async fn download_file_if_not_exists(path: &PathBuf, url: String, size: u64) -> Void {
    if !verify_file_existence(&path.to_str().unwrap().to_string(), size) {
        download_file(url, path.to_str().unwrap().to_string()).await?;
    }
    Ok(())
}

async fn download_from_manifest(id: &String, manifest: &Manifest, mir: &Mirror) -> Void {
    let version = manifest
        .versions
        .iter()
        .find(|v| &v.id == id)
        .expect(format!("Couldn't find version in manifest. {id}").as_str());
    let version_url = mir.parse_url(&version.url);
    download_file(
        version_url.to_string(),
        get_version_directory(&id)
            .join(format!("{}.json", id))
            .to_str()
            .unwrap()
            .to_string(),
    )
    .await
}

async fn download_client(
    value: &DownloadDetail,
    version: &String,
    logger: &UnboundedSender<LogLine>,
    mirror: &Mirror,
) -> Void {
    let size = value.size;
    let url = mirror.parse_url(&value.url);
    let path = get_versions_directory()
        .join(&version)
        .join(format!("{}.jar", version));
    download_file_if_not_exists(&path, url.to_string(), size).await
}

async fn download_libraries(
    libraries: &[Library],
    version: &String,
    app_handle: &AppHandle,
    logger: &UnboundedSender<LogLine>,
    mirror: &Mirror,
) -> Void {
    let libraries_path = get_libraries_directory();

    for (library_index, library) in libraries.iter().enumerate() {
        if library.downloads.is_none() {
            let name = library.name.replace(":", "/");
            let parts = name.split("/").collect::<Vec<&str>>();
            let group = parts[0].replace(".", "/");
            let artifact = parts[1];
            let version = parts[2];
            let path = format!("{group}/{artifact}/{version}/{artifact}-{version}.jar");
            if group.to_lowercase() == "net/minecraft" {
                let url = mirror.parse_url(&format!("https://libraries.minecraft.net/{path}"));
                let full_path = get_libraries_directory().join(path);
                download_file_if_not_exists(&full_path, url, 0).await?;
            } else {
                let urls = vec![
                    format!("https://maven.minecraftforge.net/{path}"),
                    format!("https://repo.spongepowered.org/maven/{path}"),
                ];
                for url in urls {
                    let full_path = get_libraries_directory().join(&path);
                    if reqwest::get(url.clone())
                        .await
                        .map_err(|x| request_unknown_err(x))?
                        .status()
                        .is_success()
                    {
                        download_file_if_not_exists(&full_path, url, 0).await?;
                    }
                }
            }
            continue;
        }

        let downloads = library.downloads.as_ref().unwrap();
        if downloads.artifact.is_none() {
            download_classifiers(downloads.classifiers.as_ref(), version, mirror).await?;
            continue;
        }
        let library_info = library_from_value(library);
        update_download(
            (library_index * 100 / libraries.len()) as i64,
            format!("Downloading {}", library_info.name).as_str(),
            app_handle,
        );
        let os = get_current_os();
        let rules = fetch_rules(library.rules.as_ref());
        download_classifiers(downloads.classifiers.as_ref(), version, mirror).await?;
        if rules.allowed_oses.contains(&os) && !rules.disallowed_oses.contains(&os) {
            let path = libraries_path.join(&library_info.path.as_str());
            download_file_if_not_exists(
                &path,
                mirror.parse_url(&library_info.url),
                library_info.size,
            )
            .await?;
        }
    }
    Ok(())
}

async fn download_classifiers(
    classifiers: Option<&HashMap<String, LibraryArtifact>>,
    version: &String,
    mirror: &Mirror,
) -> Void {
    if classifiers.is_none() {
        return Ok(());
    }
    let os = get_current_os();
    let classifiers_map = classifiers.unwrap();
    let mut natives = classifiers_map.get(&format!("natives-{os}"));
    if natives.is_none() && os == "windows" {
        natives = classifiers_map.get(&format!("natives-{os}-64"));
    }
    match natives {
        None => {}
        Some(val) => {
            let url = mirror.parse_url(&val.url.to_string());
            let url_https_less = url.replace("https://", "").replace("http://", "");
            let path = if val.path.is_none() {
                let url_args = url_https_less.split("/").collect::<Vec<&str>>();
                let path = url_https_less.replace(url_args[0], "");
                path
            } else {
                val.path.as_ref().unwrap().to_string()
            };
            let full_path = get_libraries_directory().join(path);
            let size = val.size;
            download_file_if_not_exists(&full_path, url.to_string(), size).await?;
            let file = File::open(full_path.to_str().unwrap().to_string());
            let natives_path = get_natives_folder(version);
            if !exists(&natives_path).unwrap() {
                create_dir_all(&natives_path).unwrap();
            }
            extract(file.unwrap(), &natives_path, false).unwrap();
        }
    }
    Ok(())
}

fn fetch_rules(value: Option<&Vec<Rule>>) -> LibraryRules {
    if value.is_none() {
        return LibraryRules {
            allowed_oses: vec![
                "osx".to_string(),
                "windows".to_string(),
                "linux".to_string(),
            ],
            disallowed_oses: vec![],
        };
    }
    let rules = value.unwrap();
    let mut allowed = vec![];
    let mut disallowed = vec![];
    for rule in rules {
        let rule_action = &rule.action;
        let rule_os = &rule.os;
        if rule_action == "allow" {
            if rule_os.is_none() {
                allowed.push("osx".to_string());
                allowed.push("windows".to_string());
                allowed.push("linux".to_string());
            } else {
                let os_name = rule_os.as_ref().unwrap().name.as_ref();
                if let Some(name) = os_name {
                    allowed.push(name.to_string());
                }
            }
        } else if rule_action == "disallow" {
            if rule_os.is_none() {
                disallowed.push("osx".to_string());
                disallowed.push("windows".to_string());
                disallowed.push("linux".to_string());
            } else {
                let os_name = rule_os.as_ref().unwrap().name.as_ref();
                if let Some(name) = os_name {
                    disallowed.push(name.to_string());
                }
            }
        }
    }
    LibraryRules {
        allowed_oses: allowed,
        disallowed_oses: disallowed,
    }
}

fn download_file_async(url: String, dest: String) -> Void {
    block_on(async { download_file(url, dest).await })
}
fn download_file_async_thread(url: String, dest: String) -> Void {
    block_on(async { download_file(url, dest).await })
}

pub async fn download_file(url: String, dest: String) -> Void {
    let resp = reqwest::get(&url)
        .await
        .map_err(|x| download_error(format!("Failed to download file from {url}, {}", x)))?;
    println!(
        "Downloading {url} to {dest} with response of {}",
        resp.content_length().unwrap()
    );
    let dest_folder = PathBuf::from(&dest)
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    if !exists(&dest_folder).unwrap() {
        create_dir_all(&dest_folder).expect("Creating directory failed.");
    }

    let mut out =
        File::create(&dest).expect(format!("Unable to create file. at {}", dest.as_str()).as_str());
    out.write_all(&resp.bytes().await.unwrap())
        .expect("Writing file failed.");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if let Ok(metadata) = std::fs::metadata(&dest) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755); // rwxr-xr-x
            set_permissions(&dest, permissions).map_err(|x| io_err_permission(x))?;
        }
    }
    Ok(())
}

pub async fn get_available_forge_versions(version_id: &String) -> Returns<Vec<String>> {
    let mut global_cache = GLOBAL_CACHE.lock().await;
    if global_cache.forge.is_none() {
        let url = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";
        let map: HashMap<String, Vec<String>> = reqwest::get(url)
            .await
            .map_err(|x| request_unknown_err(x))?
            .json()
            .await
            .map_err(|x| request_unknown_err(x))?;
        global_cache.forge = Some(map);
    }
    let map = &global_cache.forge;
    Ok(map
        .clone()
        .unwrap_or(HashMap::new())
        .iter()
        .find(|(key, _)| key.as_str() == version_id.as_str())
        .map(|(_key, val)| val.clone())
        .unwrap_or(Vec::new()))
}

pub async fn download_forge_version(
    version: &String,
    app_handle: &AppHandle,
    logger: &UnboundedSender<LogLine>,
    mirror: &Mirror,
) -> Void {
    let url = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{version}/forge-{version}-installer.jar");
    let launcher_dir = get_falcon_launcher_directory();

    let mut path = launcher_dir.join("temp");
    let mut path_str = path.to_str().unwrap();

    if !path.exists() {
        create_dir_all(path_str).unwrap();
    }

    path = path.join(format!("forge-{version}-installer.jar"));
    path_str = path.to_str().unwrap();
    download_file(url, path_str.to_string()).await?;
    let installer_file = File::open(path_str).unwrap();

    let version_args = version.split("-").collect::<Vec<&str>>();
    let mc_version = version_args[0];
    let mc_args = mc_version.split(".").collect::<Vec<&str>>();
    let version_mid = mc_args[1].parse::<i32>().unwrap();
    if version_mid > 12 {
        logger.send(info_launcher(
            "DEBUG: Non legacy version detected!".to_string(),
        ));
        download_java(&"jre-legacy".to_string(), &"8".to_string(), logger, mirror).await?;
        let jdk_8 = get_java("jre-legacy".to_string())?;
        let mut child = Command::new(jdk_8.get_bin_file().display().to_string())
            .arg("-jar")
            .arg(PathBuf::from(path_str).display().to_string())
            .arg("--installClient")
            .arg(get_minecraft_directory().display().to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(get_temp_directory())
            .spawn()
            .expect("Failed to install forge");
        let stderr = child.stderr.take().unwrap();
        let logger_clone = logger.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                logger_clone.send(info_launcher(format!("[stderr] {}", line)));
            }
        });

        generate_stdout(&mut child, logger);

        fs::remove_dir_all(launcher_dir.join("temp")).unwrap();

        return Ok(());
    }
    logger.send(info_launcher("DEBUG: Legacy version detected!".to_string()));

    let mut zip = ZipArchive::new(installer_file).unwrap();
    let install_profile_file = zip
        .by_name("install_profile.json")
        .expect("Failed to find install_profile.json");

    let install_profile_json: ForgeInstallProfile =
        serde_json::from_reader(install_profile_file).unwrap();

    if let Some(install_data) = &install_profile_json.install {
        let mut forge = zip.by_name(&install_data.file_path).unwrap();
        let path_maven = &install_data.path;
        let args = path_maven.split(":").collect::<Vec<&str>>();
        let group_id = args[0].replace(".", "/");
        let artifact = args[1];
        let version = args[2];
        let artifact_version = format!("{artifact}-{version}");
        let full_path = get_libraries_directory().join(format!(
            "{group_id}/{artifact}/{version}/{artifact_version}.jar"
        ));
        create_dir_all(&full_path.parent().unwrap()).expect("Failed to create the path");
        let mut file = File::create(full_path).unwrap();
        std::io::copy(&mut forge, &mut file).expect("Failed to copy files");
    }

    let version_json: ForgeVersionJsonInfo = if install_profile_json.version_info.is_none() {
        let versions_file = zip.by_name("version.json").unwrap();
        serde_json::from_reader(versions_file).unwrap()
    } else {
        install_profile_json.version_info.clone().unwrap()
    };

    let version_id = &version_json.id;
    let version_folder = get_version_directory(&version_id.to_string());
    if !version_folder.exists() {
        create_dir_all(version_folder).unwrap();
    }
    let version_json_path =
        get_version_directory(&version_id.to_string()).join(format!("{version_id}.json"));
    File::create(&version_json_path).unwrap();
    fs::write(
        version_json_path,
        serde_json::to_string(&version_json).unwrap(),
    )
    .expect("Failed to write to the forge json file.");

    if let Some(profile_libraries) = &install_profile_json.libraries {
        for library in profile_libraries {
            let library_downloads = if library.downloads.is_none() {
                library
            } else {
                // For layout convenience we can skip parsing empty artifact blocks
                // or safely bind fields here
                library
            };

            // Checking fields using the typed optional fields inside library rules:
            if let Some(downloads) = &library.downloads {
                if let Some(artifact) = &downloads.artifact {
                    let url = &artifact.url;
                    if url == "" {
                        if let Some(path) = &artifact.path {
                            let zip_path = format!("maven/{}", path);
                            let mut f = zip.by_name(&zip_path).expect("Stupid error ");
                            create_dir_all(
                                PathBuf::from(get_libraries_directory().join(path))
                                    .parent()
                                    .unwrap(),
                            )
                            .expect("Failed to create the directory");
                            let mut file =
                                File::create(get_libraries_directory().join(path)).unwrap();
                            std::io::copy(&mut f, &mut file).expect("Failed to copy files");
                        }
                        continue;
                    }

                    let full_url = if url.ends_with("/") {
                        convert_to_full_url(url.to_string(), library.name.to_string())
                    } else {
                        url.to_string()
                    };

                    let full_path = if artifact.path.is_none() {
                        convert_to_full_path(
                            get_libraries_directory().to_str().unwrap().to_string(),
                            &library.name,
                        )
                    } else {
                        artifact.path.as_ref().unwrap().to_string()
                    };

                    download_file_if_not_exists(&PathBuf::from(full_path), full_url, 0).await?;
                }
            }
        }
    }

    for library in &version_json.libraries {
        if let Some(url) = &library.url {
            let full_url = convert_to_full_url(url.to_string(), library.name.to_string());
            let full_path = convert_to_full_path(
                get_libraries_directory().to_str().unwrap().to_string(),
                &library.name,
            );

            download_file_if_not_exists(&PathBuf::from(full_path), full_url, 0).await?;
        }
    }

    fs::remove_dir_all(launcher_dir.join("temp")).unwrap();
    Ok(())
}

pub async fn download_fabric(
    version_loader: &VersionLoader,
    logger: &UnboundedSender<LogLine>,
    mirror: &Mirror,
) -> Void {
    let loaders_url = "https://meta.fabricmc.net/v2/versions/loader";
    let installers_url = "https://meta.fabricmc.net/v2/versions/installer";
    type FabricLoaders = Vec<FabricLoader>;
    type FabricInstallers = Vec<FabricInstaller>;
    let loaders = reqwest::get(loaders_url)
        .await
        .unwrap()
        .json::<FabricLoaders>()
        .await
        .unwrap();
    let installers = reqwest::get(installers_url)
        .await
        .unwrap()
        .json::<FabricInstallers>()
        .await
        .unwrap();

    let loader = loaders
        .iter()
        .find(|x| x.version == version_loader.get_fabric_loader_id())
        .unwrap();
    let stable_installer = installers.iter().find(|x| x.stable).unwrap();
    let installer_path_download = convert_to_full_path(
        get_temp_directory().to_str().unwrap().to_string(),
        &stable_installer.maven,
    );

    download_file(
        stable_installer.url.to_string(),
        installer_path_download.clone(),
    )
    .await?;
    download_java(&"jre-legacy".to_string(), &"8".to_string(), logger, mirror).await?;
    let jdk_8 = get_java("jre-legacy".to_string())?;
    let mut child = Command::new(jdk_8.get_bin_file())
        .arg("-jar")
        .arg(installer_path_download.clone())
        .arg("client")
        .arg("-mcVersion")
        .arg(version_loader.get_fabric_version_id())
        .arg("-loader")
        .arg(version_loader.get_fabric_loader_id())
        .arg("-dir")
        .arg(get_minecraft_directory().display().to_string())
        .current_dir(
            PathBuf::from(installer_path_download)
                .parent()
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");
    let stderr = child.stderr.take().unwrap();
    let logger_clone = logger.clone();
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            logger_clone.send(info_launcher(format!("[stderr] {}", line)));
        }
    });

    generate_stdout(&mut child, logger);
    Ok(())
}

pub fn generate_stdout(child: &mut Child, logger: &UnboundedSender<LogLine>) {
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let logger_clone = logger.clone();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                logger_clone.send(info_launcher(format!("[stdout] {}", line)));
            }
        }
    });
}

pub async fn get_available_fabric_versions(version_id: &String) -> Returns<Vec<String>> {
    let mut global_cache = GLOBAL_CACHE.lock().await;
    if global_cache.fabric_mc_versions.is_none() {
        let url = "https://meta.fabricmc.net/v2/versions/game";
        let map: Vec<FabricMinecraftVersion> = reqwest::get(url)
            .await
            .map_err(|x| request_unknown_err(x))?
            .json()
            .await
            .map_err(|x| request_unknown_err(x))?;
        global_cache.fabric_mc_versions = Some(map);
    }
    if global_cache.fabric_installers.is_none() {
        let url = "https://meta.fabricmc.net/v2/versions/installer";
        let map: Vec<FabricInstaller> = reqwest::get(url)
            .await
            .map_err(|x| request_unknown_err(x))?
            .json()
            .await
            .map_err(|x| request_unknown_err(x))?;
        global_cache.fabric_installers = Some(map);
    }
    if global_cache.fabric_loaders.is_none() {
        let url = "https://meta.fabricmc.net/v2/versions/loader";
        let map: Vec<FabricLoader> = reqwest::get(url)
            .await
            .map_err(|x| request_unknown_err(x))?
            .json()
            .await
            .map_err(|x| request_unknown_err(x))?;
        global_cache.fabric_loaders = Some(map);
    }

    let map = &global_cache.fabric_mc_versions;
    let unwrapped_map = map.clone().unwrap_or(Vec::new());
    let v = unwrapped_map.iter().find(|x| &x.version == version_id);
    if v.is_none() {
        return Ok(Vec::new());
    }

    let mut result = Vec::new();
    let loaders = &global_cache.fabric_loaders;
    for loader in loaders.clone().unwrap() {
        result.push(format!("{}-{}", version_id.to_string(), loader.version));
    }
    Ok(result)
}
