#![allow(deprecated)]

use crate::models::versions::MinecraftVersion;
use crate::services::directory_manager::{
    get_assets_directory, get_falcon_launcher_directory, get_libraries_directory,
    get_minecraft_directory, get_natives_folder, get_temp_directory, get_version_directory,
    get_version_manifest, get_versions_directory,
};
use crate::services::utils::{
    convert_to_full_path, convert_to_full_url, fetch_library_path, fetch_rules,
    fetch_unofficial_library_repos, is_legacy, verify_file_existence,
};
use crate::services::utils::{update_download, update_download_bar, update_download_status};
use crate::services::version_manager::load_version_manifest;

use crate::models::config::Config;
use crate::models::downloader::{
    AssetIndex, AssetObjects, DownloadDetail, ForgeInstallProfile, ForgeVersionJsonInfo, Library,
    LibraryArtifact, Manifest, MinecraftManifestVersion, VersionLoader,
};
use crate::models::error::AppError;
use crate::models::fabric::{FabricInstaller, FabricLoader, FabricMinecraftVersion};
use crate::models::logger::{LogLine};
use crate::models::mirror::Mirror;
use crate::models::platform::get_current_os;
use crate::models::utils::{LowerCaseStartsWith, ParseWithMirror};
use crate::services::jdk_manager::{download_java, get_java};
use crate::GLOBAL_CACHE;
use log::info;
use reqwest::Client;
use std::collections::HashMap;
use std::fs;
use std::fs::{create_dir_all, exists, set_permissions, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStderr, Command, Stdio};
use std::time::Duration;
use tauri::async_runtime::block_on;
use tauri::AppHandle;
use tokio::sync::mpsc::UnboundedSender;
use zip::ZipArchive;
use zip_extract::extract;

pub async fn download_version(
    version: &MinecraftVersion,
    name: &String,
    app_handle: &AppHandle,
    logger: &UnboundedSender<LogLine>,
    cfg: &Config,
) -> Result<(), AppError> {
    let id = &version.id;
    let mirror = &cfg.download_settings.mirror;
    let name = if name == "" {
        &version.id
    } else {
        name
    };

    info!("Downloading version {} with name of {name}", &version.id);

    let manifest = load_version_manifest(&mirror).await?;
    download_from_manifest(id, &manifest, &mirror)
        .await
        .or_else(|x| {
            if exists(get_version_manifest(id)).unwrap() {
                Ok(())
            } else {
                Err(x)
            }
        })?;
    let content =
        fs::read_to_string(PathBuf::from(version.get_json())).map_err(|x| AppError::FileReadFailed(x.to_string()))?;
    let json: MinecraftManifestVersion =
        serde_json::from_str(&content).map_err(|x| AppError::JsonParseFailed(x.to_string()))?;
    let java_version = if json.inherits_from.is_none() {
        &json.java_version.unwrap()
    } else {
        let dir = get_version_manifest(&json.inherits_from.unwrap().as_str().to_string());
        println!("{}", dir.display());
        let content = fs::read_to_string(dir);
        let m: MinecraftManifestVersion = serde_json::from_str(&content.unwrap()).map_err(|x| AppError::JsonParseFailed(x.to_string()))?;
        &m.java_version.unwrap()
    };
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
            info!("Downloading client's process has started.");
            update_download_status("Downloading version...", &app_handle);
            download_client(client_download, &id, logger, &mirror).await?;
        }
    }

    if let Some(asset_index) = &json.asset_index {
        info!("Downloading assets process has started.");
        update_download_status("Downloading assets...", &app_handle);
        download_assets(asset_index, logger, &mirror, app_handle).await?;
    }
    if let Some(logging) = &json.logging {
        info!("Downloading logger files process has started.");
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
) -> Result<(), AppError> {
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

pub async fn download_file_if_not_exists(path: &PathBuf, url: String, size: u64) -> Result<(), AppError> {
    if !verify_file_existence(&path.to_str().unwrap().to_string(), size) {
        download_file(url, &path.to_str().unwrap().to_string()).await?;
    }
    Ok(())
}

pub(crate) async fn download_from_manifest(id: &String, manifest: &Manifest, mir: &Mirror) -> Result<(), AppError> {
    let version = manifest
        .versions
        .iter()
        .find(|v| &v.id == id)
        .ok_or(AppError::ManifestParseFailed(
            format!("Couldn't find version in manifest. {id}")
        ))?;
    let version_url = mir.parse_url(&version.url);
    download_file(
        version_url.to_string(),
        &get_version_directory(&id)
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
) -> Result<(), AppError> {
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
) -> Result<(), AppError> {
    let libraries_path = get_libraries_directory();

    for (library_index, library) in libraries.iter().enumerate() {
        if library.downloads.is_none() {
            let name = library.name.replace(":", "/");
            let path = fetch_library_path(&name);
            if name.starts_with_lower_case("net/minecraft") {
                let url = mirror.parse_url(&format!("https://libraries.minecraft.net/{path}"));
                let full_path = get_libraries_directory().join(path);
                download_file_if_not_exists(&full_path, url, 0).await?;
            } else {
                let urls = fetch_unofficial_library_repos(&path);
                for url in urls {
                    let full_path = get_libraries_directory().join(&path);
                    let client = Client::builder()
                        .connect_timeout(Duration::from_secs(3))
                        .build()
                        .unwrap();
                    if client
                        .head(url.clone())
                        .send()
                        .await
                        .map_err(|x| AppError::NetworkRequestFailed(x.to_string()))?
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

        let library_artifact = library
            .downloads
            .as_ref()
            .and_then(|d| d.artifact.as_ref())
            .ok_or(AppError::ManifestParseFailed("Parsing library downloads failed".to_string()))?;

        let library_path = if library_artifact.path.is_none() {
            let args = library.name.split(":").collect::<Vec<&str>>();
            let group_id = args[0].replace(".", "/");
            let artifact = args[1];
            let version = args[2];
            let artifact_version = format!("{artifact}-{version}.jar");
            format!("{group_id}/{artifact}/{version}/{artifact_version}")
        } else {
            library_artifact.path.as_ref().unwrap().to_string()
        };

        update_download(
            (library_index * 100 / libraries.len()) as i64,
            format!("Downloading {}", library.name).as_str(),
            app_handle,
        );

        let os = get_current_os();
        let rules = fetch_rules(library.rules.as_ref());

        download_classifiers(downloads.classifiers.as_ref(), version, mirror).await?;

        if rules.allowed_oses.contains(&os) && !rules.disallowed_oses.contains(&os) {
            let path = libraries_path.join(&library_path.as_str());
            download_file_if_not_exists(
                &path,
                mirror.parse_url(&library_artifact.url),
                library_artifact.size,
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
) -> Result<(), AppError> {
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
            // TODO: changing natives folder to a better place
            extract(file.unwrap(), &natives_path, false)
                .map_err(|x| AppError::ZipExtractionFailed("Zip extraction of classifier failed".to_string()))?;
        }
    }
    Ok(())
}

fn download_file_async(url: String, dest: String) -> Result<(), AppError> {
    block_on(async { download_file(url, &dest).await })
}
fn download_file_async_thread(url: String, dest: String) -> Result<(), AppError> {
    block_on(async { download_file(url, &dest).await })
}

pub async fn download_file(url: String, dest: &String) -> Result<(), AppError> {
    let resp = reqwest::get(&url)
        .await
        //format!("Failed to download file from {url}, {}", x))
        .map_err(|x| AppError::DownloadFailed)?;
    info!(
        "Downloading {url} to {dest} with response of {}",
        resp.content_length().unwrap()
    );
    let dest_folder = PathBuf::from(dest)
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
            set_permissions(&dest, permissions).map_err(|x| AppError::AccessDenied(x.to_string()))?;
        }
    }
    Ok(())
}

pub async fn get_available_forge_versions(
    version_id: &String,
    mirror: &Mirror,
) -> Result<Vec<String>, AppError> {
    let mut global_cache = GLOBAL_CACHE.lock().await;
    if global_cache.forge.is_none() {
        let url = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json"
            .parse_mirror(&mirror);

        let map: HashMap<String, Vec<String>> = reqwest::get(url)
            .await
            .map_err(|x| AppError::NetworkRequestFailed(x.to_string()))?
            .json()
            .await
            .map_err(|x| AppError::NetworkRequestFailed(x.to_string()))?;
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

pub async fn fetch_forge_mirrors(content: String) -> Vec<String> {
    let mut vec = Vec::new();
    for line in content.split("\n") {
        let args = line.split("!").collect::<Vec<&str>>();
        vec.push(args[args.len() - 1].to_string());
    }
    vec
}
pub async fn download_forge_version(
    version: &String,
    app_handle: &AppHandle,
    logger: &UnboundedSender<LogLine>,
    mirror: &Mirror,
    ver: &mut String,
) -> Result<(), AppError> {
    let url = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{version}/forge-{version}-installer.jar").parse_mirror(mirror);
    let launcher_dir = get_falcon_launcher_directory();
    info!("{}", url);
    let mut path = launcher_dir.join("temp");
    let mut path_str = path.to_str().unwrap();
    info!("{}", path_str);

    if !path.exists() {
        create_dir_all(path_str).unwrap();
    }

    path = path.join(format!("forge-{version}-installer.jar"));
    path_str = path.to_str().unwrap();
    download_file(url, &path_str.to_string()).await?;

    let version_args = version.split("-").collect::<Vec<&str>>();
    let mc_version = version_args[0];
    let mc_args = mc_version.split(".").collect::<Vec<&str>>();
    if !is_legacy(&version) {
            info!(
                "DEBUG: Non legacy version detected!",
            );
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

        spawn_thread(stderr, format!("forge_installer_{version}"));

        generate_stdout(&mut child, format!("forge_installer_{version}"));
        let _ = child.wait_with_output(); // Ensuring that the forge installer.jar job is done.
        fs::remove_dir_all(launcher_dir.join("temp")).unwrap();

        return Ok(());
    }
    info!("DEBUG: Legacy version detected!");
    let installer_file = File::open(path_str).unwrap();

    let mut zip = ZipArchive::new(installer_file).unwrap();
    let install_profile_file = zip
        .by_name("install_profile.json")
        .map_err(|x| AppError::ProfileNotFound("Failed to find install_profile.json".to_string()))?;

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
        let mirror_list = mirror.parse_url(&install_data.mirror_list);
        let resp = reqwest::get(mirror_list)
            .await.map(async |x| x.text().await);
        if resp.is_ok() {
            let resp = resp.unwrap().await.map(|s| fetch_forge_mirrors(s));
            if resp.is_ok() {
                let mirrors = resp.unwrap().await;
            }
        }
        create_dir_all(&full_path.parent().unwrap())
            .map_err(|x| AppError::DirCreateFailed("Failed to create the path".to_string()))?;

        let mut file = File::create(full_path).unwrap();
        std::io::copy(&mut forge, &mut file).map_err(|x| AppError::FileCopyFailed("Failed to copy files".to_string()))?;
    }

    let version_json: ForgeVersionJsonInfo = if install_profile_json.version_info.is_none() {
        let versions_file = zip.by_name("version.json").unwrap();
        serde_json::from_reader(versions_file).unwrap()
    } else {
        install_profile_json.version_info.clone().unwrap()
    };

    let version_id = &version_json.id;
    *ver = version_id.clone();
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
        .map_err(|x| AppError::FileWriteFailed("Failed to write to the forge json file.".to_string()))?;

    if let Some(profile_libraries) = &install_profile_json.libraries {
        for library in profile_libraries {
            if let Some(downloads) = &library.downloads {
                if let Some(artifact) = &downloads.artifact {
                    let url = &artifact.url;
                    if url == "" {
                        if let Some(path) = &artifact.path {
                            let zip_path = format!("maven/{}", path);
                            let mut f = zip
                                .by_name(&zip_path)
                                .map_err(|x| AppError::ZipParseFailed("Parsing zip file failed.".to_string()))?;
                            create_dir_all(
                                PathBuf::from(get_libraries_directory().join(path))
                                    .parent()
                                    .unwrap(),
                            )
                                .map_err(|x| AppError::DirCreateFailed("Failed to create the directory".to_string()))?;
                            let mut file =
                                File::create(get_libraries_directory().join(path)).unwrap();
                            std::io::copy(&mut f, &mut file)
                                .map_err(|x| AppError::FileCopyFailed("Failed to copy files".to_string()))?;
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

fn spawn_thread(stderr: ChildStderr,task_name: String) {
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            info!("[{task_name}][stderr] {}", line);
        }
    });
}

pub async fn download_fabric(
    version_loader: &VersionLoader,
    logger: &UnboundedSender<LogLine>,
    mirror: &Mirror,
) -> Result<(), AppError> {
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
        &installer_path_download.clone(),
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
    spawn_thread(stderr, format!("fabric_installer_{}",version_loader.id));

    generate_stdout(&mut child, format!("fabric_installer_{}",version_loader.id));
    Ok(())
}

pub fn generate_stdout(child: &mut Child, task_name: String) {
    let stdout = child.stdout.take().expect("Failed to open stdout");
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                info!("[{task_name}][stdout] {}", line);
            }
        }
    });
}

pub async fn get_available_fabric_versions(version_id: &String) -> Result<Vec<String>, AppError> {
    let mut global_cache = GLOBAL_CACHE.lock().await;
    if global_cache.fabric_mc_versions.is_none() {
        let url = "https://meta.fabricmc.net/v2/versions/game";
        let map: Vec<FabricMinecraftVersion> = reqwest::get(url)
            .await
            .map_err(|x| AppError::NetworkRequestFailed(x.to_string()))?
            .json()
            .await
            .map_err(|x| AppError::NetworkRequestFailed(x.to_string()))?;
        global_cache.fabric_mc_versions = Some(map);
    }
    if global_cache.fabric_installers.is_none() {
        let url = "https://meta.fabricmc.net/v2/versions/installer";
        let map: Vec<FabricInstaller> = reqwest::get(url)
            .await
            .map_err(|x| AppError::NetworkRequestFailed(x.to_string()))?
            .json()
            .await
            .map_err(|x| AppError::NetworkRequestFailed(x.to_string()))?;
        global_cache.fabric_installers = Some(map);
    }
    if global_cache.fabric_loaders.is_none() {
        let url = "https://meta.fabricmc.net/v2/versions/loader";
        let map: Vec<FabricLoader> = reqwest::get(url)
            .await
            .map_err(|x| AppError::NetworkRequestFailed(x.to_string()))?
            .json()
            .await
            .map_err(|x| AppError::NetworkRequestFailed(x.to_string()))?;
        global_cache.fabric_loaders = Some(map);
    }

    let map = &global_cache.fabric_mc_versions;
    let unwrapped_map = map.clone().unwrap_or(Vec::new());
    let v = unwrapped_map.iter().find(|x| &x.version == version_id);
    if v.is_none() {
        return Ok(Vec::new());
    }

    let mut result = Vec::new();
    let loaders = &global_cache.fabric_loaders.clone().unwrap_or(Vec::new());
    for loader in loaders {
        result.push(format!("{}-{}", version_id.to_string(), loader.version));
    }
    Ok(result)
}
