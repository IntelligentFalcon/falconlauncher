#![allow(deprecated)]

use crate::models::versions::MinecraftVersion;
use crate::services::directory_manager::{
    get_assets_directory, get_falcon_launcher_directory, get_libraries_directory,
    get_minecraft_directory, get_natives_folder, get_temp_directory, get_version_directory,
    get_version_manifest, get_versions_directory,
};
use crate::services::utils::{
    convert_to_full_path, convert_to_full_url, fetch_library_path, fetch_rules,
    fetch_unofficial_library_repos, is_legacy, verify_file_existence_with_sha,
    verify_file_existence_with_size,
};
use crate::services::utils::{update_download, update_download_bar, update_download_status};
use crate::services::version_manager::{load_version_manifest, load_version_manifest_local};

use crate::models::config::Config;
use crate::models::downloader::{
    AssetIndex, AssetObjects, DownloadDetail, ForgeInstallProfile, ForgeVersionJsonInfo, Library,
    LibraryArtifact, Manifest, MinecraftManifestVersion, VersionLoader,
};
use crate::models::error::AppError;
use crate::models::fabric::{FabricInstaller, FabricLoader, FabricMinecraftVersion};
use crate::models::logger::LogLine;
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
use std::path::{Path, PathBuf};
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
    let name = if name == "" { &version.id } else { name };

    info!("Downloading version {} with name of {name}", &version.id);

    let manifest = load_version_manifest(&mirror).await?;
    download_from_manifest(id, &manifest, &mirror)
        .await
        .or_else(|x| {
            if get_version_manifest(id).exists() {
                Ok(())
            } else {
                Err(x)
            }
        })?;

    let content = fs::read_to_string(PathBuf::from(version.get_json()))
        .map_err(|x| AppError::FileReadFailed(format!("Failed to read version JSON: {}", x)))?;
    let json: MinecraftManifestVersion =
        serde_json::from_str(&content).map_err(|x| AppError::JsonParseFailed(x.to_string()))?;

    let java_version = if let Some(jv) = &json.java_version {
        jv.clone()
    } else if let Some(inherits) = &json.inherits_from {
        let dir = get_version_manifest(inherits);
        let content = fs::read_to_string(dir)
            .map_err(|x| AppError::FileReadFailed(format!("Failed to read inherited JSON: {}", x)))?;
        let m: MinecraftManifestVersion = serde_json::from_str(&content)
            .map_err(|x| AppError::JsonParseFailed(x.to_string()))?;

        m.java_version
            .ok_or_else(|| AppError::ManifestParseFailed("Java version missing in inherited manifest".to_string()))?
            .clone()
    } else {
        return Err(AppError::ManifestParseFailed("No Java version found in manifest".to_string()));
    };

    download_java(
        &java_version.component,
        &java_version.major_version.to_string(),
        logger,
        &mirror,
    ).await?;

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
        let filename = logging.client.file.url.split("/").last().unwrap_or("logging.xml");
        download_file_if_not_exists(
            &get_version_directory(id).join(filename),
            logging.client.file.url.clone(),
            logging.client.file.sha1.as_str(),
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
    let hash = value.sha1.as_str();
    let asset_index_path = get_assets_directory()
        .join("indexes")
        .join(format!("{id}.json"))
        .to_string_lossy()
        .into_owned();

    download_file_if_not_exists(
        &PathBuf::from(&asset_index_path),
        url.to_string(),
        hash,
        total_size,
    ).await?;

    let content = fs::read_to_string(PathBuf::from(&asset_index_path))
        .map_err(|e| AppError::FileReadFailed(format!("Failed to read asset index: {}", e)))?;

    let json: AssetObjects = serde_json::from_str(content.as_str())
        .map_err(|e| AppError::JsonParseFailed(format!("Asset index isn't well formatted: {}", e)))?;

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
        download_file_if_not_exists(&path, url, hash.as_str(), size).await?;
    }
    Ok(())
}

pub async fn download_file_if_not_exists(
    path: &PathBuf,
    url: String,
    hash: &str,
    size: u64,
) -> Result<(), AppError> {
    let path_str = path.to_string_lossy().into_owned();
    if hash != "" {
        info!("Found sha1 let's go!");
        if !verify_file_existence_with_sha(path, hash)? {
            info!("Invalid hash was found!");
            download_file(url, &path_str).await?;
        }
        return Ok(());
    }
    if !verify_file_existence_with_size(&path_str, size)? {
        download_file(url, &path_str).await?;
    }
    Ok(())
}

pub(crate) async fn download_from_manifest(
    id: &String,
    manifest: &Manifest,
    mir: &Mirror,
) -> Result<(), AppError> {
    let version = manifest
        .versions
        .iter()
        .find(|v| &v.id == id)
        .ok_or_else(|| AppError::ManifestParseFailed(format!("Couldn't find version in manifest. {id}")))?;

    let version_url = mir.parse_url(&version.url);
    let dest = get_version_directory(id)
        .join(format!("{}.json", id))
        .to_string_lossy()
        .into_owned();

    download_file(version_url.to_string(), &dest).await
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
    let hash = value.sha1.as_str();
    download_file_if_not_exists(&path, url.to_string(), hash, size).await
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
            let path = fetch_library_path(&name)?;
            if name.starts_with_lower_case("net/minecraft") {
                let url = mirror.parse_url(&format!("https://libraries.minecraft.net/{path}"));
                let full_path = get_libraries_directory().join(path);
                download_file_if_not_exists(&full_path, url, "", 0).await?;
            } else {
                let urls = fetch_unofficial_library_repos(&path);
                for url in urls {
                    let full_path = get_libraries_directory().join(&path);
                    let client = Client::builder()
                        .connect_timeout(Duration::from_secs(3))
                        .build()
                        .map_err(|e| AppError::Internal(e.to_string()))?;

                    if client
                        .head(url.clone())
                        .send()
                        .await
                        .map_err(|x| AppError::NetworkRequestFailed(x.to_string()))?
                        .status()
                        .is_success()
                    {
                        download_file_if_not_exists(&full_path, url, "", 0).await?;
                    }
                }
            }
            continue;
        }

        let downloads = library.downloads.as_ref().ok_or_else(|| AppError::ManifestParseFailed("Downloads missing".to_string()))?;
        if downloads.artifact.is_none() {
            download_classifiers(downloads.classifiers.as_ref(), version, mirror).await?;
            continue;
        }

        let library_artifact = downloads
            .artifact
            .as_ref()
            .ok_or_else(|| AppError::ManifestParseFailed("Parsing library downloads failed".to_string()))?;

        let library_path = if let Some(path) = &library_artifact.path {
            path.to_string()
        } else {
            let args = library.name.split(":").collect::<Vec<&str>>();
            if args.len() < 3 {
                return Err(AppError::ManifestParseFailed(format!("Invalid library name: {}", library.name)));
            }
            let group_id = args[0].replace(".", "/");
            let artifact = args[1];
            let version = args[2];
            format!("{group_id}/{artifact}/{version}/{artifact}-{version}.jar")
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
            let hash = &library_artifact.sha1.clone().unwrap_or_default();
            download_file_if_not_exists(
                &path,
                mirror.parse_url(&library_artifact.url),
                hash.as_str(),
                library_artifact.size.unwrap_or(0),
            ).await?;
        }
    }
    Ok(())
}

async fn download_classifiers(
    classifiers: Option<&HashMap<String, LibraryArtifact>>,
    version: &String,
    mirror: &Mirror,
) -> Result<(), AppError> {
    let classifiers_map = match classifiers {
        Some(c) => c,
        None => return Ok(()),
    };

    let os = get_current_os();
    let mut natives = classifiers_map.get(&format!("natives-{os}"));
    if natives.is_none() && os == "windows" {
        natives = classifiers_map.get(&format!("natives-{os}-64"));
    }

    if let Some(val) = natives {
        let url = mirror.parse_url(&val.url.to_string());
        let url_https_less = url.replace("https://", "").replace("http://", "");
        let path = if let Some(p) = &val.path {
            p.to_string()
        } else {
            let url_args = url_https_less.split("/").collect::<Vec<&str>>();
            url_https_less.replace(url_args[0], "")
        };

        let full_path = get_libraries_directory().join(path);
        let size = val.size.unwrap_or(0);
        let hash = val.sha1.clone().unwrap_or_default();
        download_file_if_not_exists(&full_path, url.to_string(), hash.as_str(), size).await?;

        let file_path = full_path.to_string_lossy().into_owned();
        let file = File::open(&file_path).map_err(|e| AppError::FileReadFailed(format!("Failed to open classifier: {}", e)))?;
        let natives_path = get_natives_folder(version);

        if !exists(&natives_path).unwrap_or(false) {
            create_dir_all(&natives_path).map_err(|e| AppError::DirCreateFailed(e.to_string()))?;
        }

        extract(file, &natives_path, false)
            .map_err(|_| AppError::ZipExtractionFailed("Zip extraction of classifier failed".to_string()))?;
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
        .map_err(|_| AppError::DownloadFailed)?;

    info!(
        "Downloading {url} to {dest} with response of {}",
        resp.content_length().unwrap_or(0)
    );

    let dest_folder = PathBuf::from(dest)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    if !dest_folder.is_empty() && !exists(&dest_folder).unwrap_or(false) {
        create_dir_all(&dest_folder).map_err(|e| AppError::DirCreateFailed(e.to_string()))?;
    }

    let mut out = File::create(&dest).map_err(|e| AppError::FileCreateFailed(format!("Unable to create file at {}: {}", dest, e)))?;
    let bytes = resp.bytes().await.map_err(|e| AppError::DownloadFailed)?;
    out.write_all(&bytes).map_err(|e| AppError::FileWriteFailed(e.to_string()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&dest) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755); // rwxr-xr-x
            set_permissions(&dest, permissions)
                .map_err(|x| AppError::AccessDenied(x.to_string()))?;
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
        .unwrap_or_default()
        .get(version_id)
        .cloned()
        .unwrap_or_default())
}

pub async fn fetch_forge_mirrors(content: String) -> Vec<String> {
    let mut vec = Vec::new();
    for line in content.split("\n") {
        if line.is_empty() { continue; }
        let args = line.split("!").collect::<Vec<&str>>();
        if let Some(last) = args.last() {
            vec.push(last.to_string());
        }
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
    let mut path_str = path.to_string_lossy().into_owned();
    info!("{}", path_str);

    if !path.exists() {
        create_dir_all(&path_str).map_err(|e| AppError::DirCreateFailed(e.to_string()))?;
    }

    path = path.join(format!("forge-{version}-installer.jar"));
    path_str = path.to_string_lossy().into_owned();
    download_file(url, &path_str).await?;

    let version_args = version.split("-").collect::<Vec<&str>>();
    let mc_version = version_args.first().ok_or_else(|| AppError::ManifestParseFailed("Invalid Forge version format".to_string()))?;
    let mc_args = mc_version.split(".").collect::<Vec<&str>>();

    if !is_legacy(&version) {
        info!("DEBUG: Non legacy version detected!",);
        download_java(&"jre-legacy".to_string(), &"8".to_string(), logger, mirror).await?;
        let jdk_8 = get_java("jre-legacy".to_string())?;

        let mut child = Command::new(jdk_8.get_bin_file().display().to_string())
            .arg("-jar")
            .arg(&path_str)
            .arg("--installClient")
            .arg(get_minecraft_directory().display().to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(get_temp_directory())
            .spawn()
            .map_err(|e| AppError::Internal(format!("Failed to spawn Forge installer: {}", e)))?;

        let stderr = child.stderr.take().ok_or_else(|| AppError::Internal("Failed to capture stderr".to_string()))?;
        spawn_thread(stderr, format!("forge_installer_{version}"));
        generate_stdout(&mut child, format!("forge_installer_{version}"))?;

        let _ = child.wait_with_output();
        fs::remove_dir_all(launcher_dir.join("temp")).map_err(|e| AppError::FileDeleteFailed(e.to_string()))?;

        return Ok(());
    }

    info!("DEBUG: Legacy version detected!");
    let installer_file = File::open(&path_str).map_err(|e| AppError::FileReadFailed(e.to_string()))?;
    let mut zip = ZipArchive::new(installer_file).map_err(|e| AppError::ZipParseFailed(e.to_string()))?;

    let install_profile_file = zip.by_name("install_profile.json").map_err(|_| {
        AppError::ProfileNotFound("Failed to find install_profile.json".to_string())
    })?;

    let install_profile_json: ForgeInstallProfile =
        serde_json::from_reader(install_profile_file).map_err(|e| AppError::JsonParseFailed(e.to_string()))?;

    if let Some(install_data) = &install_profile_json.install {
        let mut forge = zip.by_name(&install_data.file_path).map_err(|e| AppError::ZipParseFailed(e.to_string()))?;
        let path_maven = &install_data.path;
        let args = path_maven.split(":").collect::<Vec<&str>>();
        if args.len() < 3 {
            return Err(AppError::ManifestParseFailed("Invalid maven path in forge installer".to_string()));
        }
        let group_id = args[0].replace(".", "/");
        let artifact = args[1];
        let version_str = args[2];
        let artifact_version = format!("{artifact}-{version_str}");
        let full_path = get_libraries_directory().join(format!(
            "{group_id}/{artifact}/{version_str}/{artifact_version}.jar"
        ));

        let mirror_list = mirror.parse_url(&install_data.mirror_list);
        if let Ok(resp) = reqwest::get(mirror_list).await {
            if let Ok(text) = resp.text().await {
                let _mirrors = fetch_forge_mirrors(text).await;
            }
        }

        create_dir_all(&full_path.parent().unwrap_or(Path::new("")))
            .map_err(|_| AppError::DirCreateFailed("Failed to create the path".to_string()))?;

        let mut file = File::create(&full_path).map_err(|e| AppError::FileCreateFailed(e.to_string()))?;
        std::io::copy(&mut forge, &mut file)
            .map_err(|_| AppError::FileCopyFailed("Failed to copy files".to_string()))?;
    }

    let version_json: ForgeVersionJsonInfo = if install_profile_json.version_info.is_none() {
        let versions_file = zip.by_name("version.json").map_err(|e| AppError::ZipParseFailed(e.to_string()))?;
        serde_json::from_reader(versions_file).map_err(|e| AppError::JsonParseFailed(e.to_string()))?
    } else {
        install_profile_json.version_info.clone().unwrap() // Already checked is_none
    };

    let version_id = &version_json.id;
    *ver = version_id.clone();
    let version_folder = get_version_directory(&version_id.to_string());

    if !version_folder.exists() {
        create_dir_all(version_folder).map_err(|e| AppError::DirCreateFailed(e.to_string()))?;
    }

    let version_json_path = get_version_directory(&version_id.to_string()).join(format!("{version_id}.json"));
    let _ = File::create(&version_json_path).map_err(|e| AppError::FileCreateFailed(e.to_string()))?;

    let json_str = serde_json::to_string(&version_json).map_err(|e| AppError::JsonParseFailed(e.to_string()))?;
    fs::write(version_json_path, json_str)
        .map_err(|_| AppError::FileWriteFailed("Failed to write to the forge json file.".to_string()))?;

    if let Some(profile_libraries) = &install_profile_json.libraries {
        for library in profile_libraries {
            if let Some(downloads) = &library.downloads {
                if let Some(artifact) = &downloads.artifact {
                    let url = &artifact.url;
                    if url == "" {
                        if let Some(path) = &artifact.path {
                            let zip_path = format!("maven/{}", path);
                            let mut f = zip.by_name(&zip_path).map_err(|_| {
                                AppError::ZipParseFailed("Parsing zip file failed.".to_string())
                            })?;

                            let lib_path = get_libraries_directory().join(path);
                            create_dir_all(lib_path.parent().unwrap_or(Path::new("")))
                                .map_err(|_| {
                                    AppError::DirCreateFailed("Failed to create the directory".to_string())
                                })?;

                            let mut file = File::create(&lib_path).map_err(|e| AppError::FileCreateFailed(e.to_string()))?;
                            std::io::copy(&mut f, &mut file).map_err(|_| {
                                AppError::FileCopyFailed("Failed to copy files".to_string())
                            })?;
                        }
                        continue;
                    }

                    let full_url = if url.ends_with("/") {
                        convert_to_full_url(url.to_string(), library.name.to_string())?
                    } else {
                        url.to_string()
                    };

                    let full_path = if artifact.path.is_none() {
                        convert_to_full_path(
                            get_libraries_directory().to_string_lossy().into_owned(),
                            &library.name,
                        )?
                    } else {
                        artifact.path.as_ref().unwrap().to_string()
                    };

                    let hash = artifact.sha1.clone().unwrap_or_default();
                    let size = artifact.size.unwrap_or_default();
                    download_file_if_not_exists(
                        &PathBuf::from(full_path),
                        full_url,
                        hash.as_str(),
                        size,
                    )
                        .await?;
                }
            }
        }
    }

    for library in &version_json.libraries {
        if let Some(url) = &library.url {
            let full_url = convert_to_full_url(url.to_string(), library.name.to_string())?;
            let full_path = convert_to_full_path(
                get_libraries_directory().to_string_lossy().into_owned(),
                &library.name,
            )?;
            download_file_if_not_exists(&PathBuf::from(full_path), full_url, "", 0).await?;
        }
    }

    fs::remove_dir_all(launcher_dir.join("temp")).map_err(|e| AppError::FileDeleteFailed(e.to_string()))?;
    Ok(())
}

fn spawn_thread(stderr: ChildStderr, task_name: String) {
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

    let loaders = reqwest::get(loaders_url)
        .await
        .map_err(|e| AppError::NetworkRequestFailed(e.to_string()))?
        .json::<Vec<FabricLoader>>()
        .await
        .map_err(|e| AppError::NetworkRequestFailed(e.to_string()))?;

    let installers = reqwest::get(installers_url)
        .await
        .map_err(|e| AppError::NetworkRequestFailed(e.to_string()))?
        .json::<Vec<FabricInstaller>>()
        .await
        .map_err(|e| AppError::NetworkRequestFailed(e.to_string()))?;

    let _loader = loaders
        .iter()
        .find(|x| x.version == version_loader.get_fabric_loader_id())
        .ok_or_else(|| AppError::VersionNotFound)?;

    let stable_installer = installers
        .iter()
        .find(|x| x.stable)
        .ok_or_else(|| AppError::VersionNotFound)?;

    let installer_path_download = convert_to_full_path(
        get_temp_directory().to_string_lossy().into_owned(),
        &stable_installer.maven,
    )?;

    download_file(
        stable_installer.url.to_string(),
        &installer_path_download,
    ).await?;

    download_java(&"jre-legacy".to_string(), &"8".to_string(), logger, mirror).await?;
    let jdk_8 = get_java("jre-legacy".to_string())?;

    let installer_path_buf = PathBuf::from(&installer_path_download);
    let current_dir = installer_path_buf
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let mut child = Command::new(jdk_8.get_bin_file())
        .arg("-jar")
        .arg(&installer_path_download)
        .arg("client")
        .arg("-mcVersion")
        .arg(version_loader.get_fabric_version_id())
        .arg("-loader")
        .arg(version_loader.get_fabric_loader_id())
        .arg("-dir")
        .arg(get_minecraft_directory().display().to_string())
        .current_dir(current_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Internal(format!("Failed to spawn Fabric installer: {}", e)))?;

    let stderr = child.stderr.take().ok_or_else(|| AppError::Internal("Failed to open stderr".to_string()))?;
    spawn_thread(stderr, format!("fabric_installer_{}", version_loader.id));

    generate_stdout(
        &mut child,
        format!("fabric_installer_{}", version_loader.id),
    )?;
    Ok(())
}

pub fn generate_stdout(child: &mut Child, task_name: String) -> Result<(), AppError> {
    let stdout = child.stdout.take().ok_or_else(|| AppError::Internal("Failed to open stdout".to_string()))?;
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            info!("[{task_name}][stdout] {}", line);
        }
    });
    Ok(())
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
    let unwrapped_map = map.clone().unwrap_or_default();

    if !unwrapped_map.iter().any(|x| &x.version == version_id) {
        return Ok(Vec::new());
    }

    let mut result = Vec::new();
    let loaders = global_cache.fabric_loaders.clone().unwrap_or_default();
    for loader in loaders {
        result.push(format!("{}-{}", version_id, loader.version));
    }

    Ok(result)
}