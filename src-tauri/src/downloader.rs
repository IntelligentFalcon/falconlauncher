use crate::directory_manager::{
    get_assets_directory, get_falcon_launcher_directory, get_libraries_directory,
    get_minecraft_directory, get_natives_folder, get_temp_directory, get_version_directory,
    get_versions_directory,
};
use crate::game_launcher::{update_download, update_download_status};
use crate::structs::{library_from_value, LibraryRules, MinecraftVersion};
use crate::utils::{
    convert_to_full_path, convert_to_full_url, get_current_os,
    verify_file_existence,
};
use crate::version_manager::{load_version_manifest, VersionLoader};

use crate::jdk_manager::get_java;
use crate::structs::fabric::{FabricInstaller, FabricLoader, FabricMinecraftVersion};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::fs::{create_dir_all, exists, File};
use std::io::{read_to_string, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use tauri::async_runtime::block_on;
use tauri::AppHandle;
use tokio::sync::Mutex;
use zip::ZipArchive;
use zip_extract::extract;

pub static GLOBAL_CACHE: LazyLock<Mutex<Global>> = LazyLock::new(|| {
    Mutex::new(Global {
        forge: None,
        fabric_loaders: None,
        fabric_installers: None,
        fabric_mc_versions: None,
    })
});
pub struct Global {
    pub forge: Option<HashMap<String, Vec<String>>>,
    pub fabric_loaders: Option<Vec<FabricLoader>>,
    pub fabric_installers: Option<Vec<FabricInstaller>>,
    pub fabric_mc_versions: Option<Vec<FabricMinecraftVersion>>,
}

async fn download_assets(value: &Value) {
    let id = value["id"].as_str().unwrap();
    let url = value["url"].as_str().unwrap();
    let total_size = value["totalSize"].as_u64().unwrap();
    let size = value["size"].as_u64().unwrap();
    let mut json: Option<Value> = None;
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
    .await;
    let content = fs::read_to_string(PathBuf::from(&asset_index_path))
        .expect("Failed to read file.");
    json = Some(serde_json::from_str(content.as_str()).expect("JSON File isn't well formatted."));
    let url_template = "https://resources.download.minecraft.net/{id}/{hash}";
    match json {
        Some(val) => {
            for (name, asset_object) in val["objects"].as_object().unwrap() {
                let hash = asset_object["hash"].as_str().unwrap();
                let id = hash[0..2].to_string().clone();
                let size = asset_object["size"].as_u64().unwrap();
                let url = url_template
                    .replace("{id}", id.as_str())
                    .replace("{hash}", hash)
                    .clone();
                let path = get_assets_directory()
                    .join("objects")
                    .join(id.as_str())
                    .join(hash);
                download_file_if_not_exists(&path, url, size).await;
            }
        }
        None => {}
    }
}

pub async fn download_file_if_not_exists(path: &PathBuf, url: String, size: u64) {
    if !verify_file_existence(&path.to_str().unwrap().to_string(), size) {
        download_file(url, path.to_str().unwrap().to_string()).await;
    }
}
pub async fn download_version(version: &MinecraftVersion, app_handle: &AppHandle) {
    let id = &version.id;

    let manifest = load_version_manifest().await;
        match manifest {
            None => {}
            Some(val) => {
                download_from_manifest(id, val).await;
            }
        }
    let content = fs::read_to_string(PathBuf::from(version.get_json())).unwrap();

    let json: Value = serde_json::from_str(&content).unwrap();

    download_libraries(&json["libraries"], &id, app_handle).await;
    if !json.get("downloads").is_none() {
        update_download_status("Downloading version...", &app_handle);
        download_client(&json["downloads"]["client"], &id).await;

    }
    if json.get("assetIndex").is_some(){
        update_download_status("Downloading assets...", &app_handle);
        download_assets(&json["assetIndex"]).await;

    }
}

async fn download_from_manifest(id: &String, manifest: Value) {
    let version = manifest["versions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["id"].as_str().unwrap() == id)
        .expect(format!("Couldn't find version in manifest. {id}").as_str());
    let version_url = version["url"].as_str().unwrap();
    download_file(
        version_url.to_string(),
        get_version_directory(&id)
            .join(format!("{}.json", id))
            .to_str()
            .unwrap()
            .to_string(),
    )
    .await;

}

async fn download_client(value: &Value, version: &String) {
    let size = value["size"].as_u64().unwrap();
    let url = value["url"].as_str().unwrap();
    let path = get_versions_directory()
        .join(&version)
        .join(format!("{}.jar", version));
    download_file_if_not_exists(&path, url.to_string(), size).await;
}

async fn download_libraries(libraries: &Value, version: &String, app_handle: &AppHandle) {
    let libraries_path = get_libraries_directory();
    let array = libraries.as_array().unwrap();
    for library_index in 0..array.len() {
        let library = &array[library_index];
        if library.get("downloads").is_none() {
            let name = library["name"].as_str().unwrap().replace(":", "/");
            let parts = name.split("/").collect::<Vec<&str>>();
            let group = parts[0].replace(".", "/");
            let artifact = parts[1];
            let version = parts[2];
            let path = format!("{group}/{artifact}/{version}/{artifact}-{version}.jar");
            if group.to_lowercase() == "net/minecraft" {
                let url = format!("https://libraries.minecraft.net/{path}");
                let full_path = get_libraries_directory().join(path);
                download_file_if_not_exists(&full_path, url, 0).await;
            } else {
                let urls = vec![
                    format!("https://maven.minecraftforge.net/{path}"),
                    format!("https://repo.spongepowered.org/maven/{path}"),
                ];
                for url in urls {
                    let full_path = get_libraries_directory().join(&path);
                    if reqwest::get(url.clone())
                        .await
                        .unwrap()
                        .status()
                        .is_success()
                    {
                        download_file_if_not_exists(&full_path, url, 0).await;
                    }
                }
            }
            continue;
        }
        if library["downloads"].get("artifact").is_none() {
            download_classifiers(library["downloads"].get("classifiers"), version).await;
            continue;
        }
        let library_info = library_from_value(library);
        update_download(
            (library_index / array.len() * 100) as i64,
            format!("Downloading {}", library_info.name).as_str(),
            app_handle,
        );
        let os = get_current_os();
        let rules = fetch_rules(library.get("rules"));
        download_classifiers(library["downloads"].get("classifiers"), version).await;
        if rules.allowed_oses.contains(&os) && !rules.disallowed_oses.contains(&os) {
            let path = libraries_path.join(&library_info.path.as_str());
            download_file_if_not_exists(&path, library_info.url, library_info.size).await;
        }
    }
}
async fn download_classifiers(classifiers: Option<&Value>, version: &String) {
    if classifiers.is_none() {
        return;
    }
    let os = get_current_os();
    let mut natives = classifiers.unwrap().get(format!("natives-{os}"));
    if natives.is_none() && os == "windows" {
        natives = classifiers.unwrap().get(format!("natives-{os}-64"));
    }
    match natives {
        None => {}
        Some(val) => {
            let url = val["url"].as_str().unwrap();
            let url_https_less = url.replace("https://", "").replace("http://", "");
            let path = if val.get("path").is_none() {
                let url_args = url_https_less.split("/").collect::<Vec<&str>>();
                let path = url_https_less.replace(url_args[0], "");
                path
            } else {
                val["path"].as_str().unwrap().to_string()
            };
            let full_path = get_libraries_directory().join(path);
            let size = val["size"].as_u64().unwrap();
            download_file_if_not_exists(&full_path, url.to_string(), size).await;
            let file = File::open(full_path.to_str().unwrap().to_string());
            let natives_path = get_natives_folder(version);
            if !exists(&natives_path).unwrap() {
                create_dir_all(&natives_path).unwrap();
            }
            extract(file.unwrap(), &natives_path, false).unwrap();
        }
    }
}
/// Fetches the rules of library which is optional
fn fetch_rules(value: Option<&Value>) -> LibraryRules {
    if value.is_none() || value.unwrap().is_null() {
        return LibraryRules {
            allowed_oses: vec![
                "osx".to_string(),
                "windows".to_string(),
                "linux".to_string(),
            ],
            disallowed_oses: vec![],
        };
    }
    let value = value.unwrap();
    let rules = value.as_array().unwrap();
    let mut allowed = vec![];
    let mut disallowed = vec![];
    for rule in rules {
        let rule_action = rule["action"].as_str().unwrap();
        let rule_os = &rule["os"]["name"];
        if rule_action == "allow" {
            if rule_os.is_null() {
                allowed.push("osx".to_string());
                allowed.push("windows".to_string());
                allowed.push("linux".to_string());
            } else {
                allowed.push(rule_os.as_str().unwrap().to_string());
            }
        } else if rule_action == "disallow" {
            if rule_os.is_null() {
                disallowed.push("osx".to_string());
                disallowed.push("windows".to_string());
                disallowed.push("linux".to_string());
            } else {
                disallowed.push(rule_os.as_str().unwrap().to_string());
            }
        }
    }
    LibraryRules {
        allowed_oses: allowed,
        disallowed_oses: disallowed,
    }
}
/// Basically download_file function without needing await.
/// uses the block_on function that causes the program to stop until the download is finished.
/// Use download_file_async_thread if you want program continue while downloading.
fn download_file_async(url: String, dest: String) {
    block_on(async {
        download_file(url, dest).await;
    })
}
fn download_file_async_thread(url: String, dest: String) {
    block_on(async {
        download_file(url, dest).await;
    });
}

pub async fn download_file(url: String, dest: String) {
    let mut resp = reqwest::get(&url)
        .await
        .expect(&format!("Downloading file failed. {url}").to_string());
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
}
pub async fn get_available_forge_versions(version_id: &String) -> Vec<String> {
    let mut global_cache = GLOBAL_CACHE.lock().await;
    if global_cache.forge.is_none() {
        let url = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";
        let map: HashMap<String, Vec<String>> =
            reqwest::get(url).await.unwrap().json().await.unwrap();
        global_cache.forge = Some(map);
    }
    let map = &global_cache.forge;
    map.clone()
        .unwrap()
        .iter()
        .find(|(key, _)| key.clone() == version_id)
        .map(|(key, val)| val.clone())
        .unwrap_or(Vec::new())
}

pub async fn download_forge_version(version: &String, app_handle: &AppHandle) {
    let url = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{version}/forge-{version}-installer.jar");
    let launcher_dir = get_falcon_launcher_directory();

    let mut path = launcher_dir.join("temp");
    let mut path_str = path.to_str().unwrap();

    if !path.exists() {
        create_dir_all(path_str).unwrap();
    }

    path = path.join(format!("forge-{version}-installer.jar"));
    path_str = path.to_str().unwrap();
    download_file(url, path_str.to_string()).await;
    let installer_file = File::open(path_str).unwrap();

    let version_args = version.split("-").collect::<Vec<&str>>();
    let mc_version = version_args[0];
    let mc_args = mc_version.split(".").collect::<Vec<&str>>();
    let version_mid = mc_args[1].parse::<i32>().unwrap();
    if version_mid > 12 {
        println!("DEBUG: Non legacy version detected!");
        let jdk_8 = get_java("8".to_string());
        let mut child = Command::new(jdk_8.await.display().to_string())
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
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                println!("[stderr] {}", line);
            }
        });

        let stdout = child.stdout.take().expect("Failed to open stdout");
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("[java stdout] {}", line);
                }
            }
        });
        fs::remove_dir_all(launcher_dir.join("temp")).unwrap();

        return;
    }
    println!("DEBUG: Legacy version detected!");

    let mut zip = ZipArchive::new(installer_file).unwrap();
    let install_profile_file = zip
        .by_name("install_profile.json")
        .expect("Failed to find install_profile.json");
    let install_profile_json: Value = serde_json::from_reader(install_profile_file).unwrap();
    if !install_profile_json.get("install").is_none()
        && !install_profile_json["install"].get("filePath").is_none()
    {
        let mut forge = zip
            .by_name(
                install_profile_json["install"]["filePath"]
                    .as_str()
                    .unwrap(),
            )
            .unwrap();
        let path_maven = install_profile_json["install"]["path"].as_str().unwrap();
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
    let version_json = if install_profile_json.get("versionInfo").is_none() {
        let versions_file = zip.by_name("version.json").unwrap();
        &serde_json::from_reader(versions_file).unwrap()
    } else {
        &install_profile_json["versionInfo"]
    };

    let version_id = version_json["id"].as_str().unwrap();
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
    if !install_profile_json.get("libraries").is_none() {
        for library in install_profile_json["libraries"].as_array().unwrap() {
            let library_downloads = if library.get("downloads").is_none() {
                &library
            } else {
                &library["downloads"]["artifact"]
            };
            let url = library_downloads["url"].as_str().unwrap();
            if url == "" {
                let path = library_downloads["path"].as_str().unwrap();
                let zip_path = format!("maven/{}", library_downloads["path"].as_str().unwrap());
                let mut f = zip.by_name(&zip_path).expect("Stupid error ");
                create_dir_all(
                    PathBuf::from(get_libraries_directory().join(&path))
                        .parent()
                        .unwrap(),
                )
                .expect("Failed to create the directory");
                let mut file = File::create(get_libraries_directory().join(&path)).unwrap();
                std::io::copy(&mut f, &mut file).expect("Failed to copy files");

                continue;
            }

            let full_url = if url.ends_with("/") {
                convert_to_full_url(
                    url.to_string(),
                    library["name"].as_str().unwrap().to_string(),
                )
            } else {
                url.to_string()
            };
            let full_path = if library_downloads.get("path").is_none() {
                convert_to_full_path(
                    get_libraries_directory().to_str().unwrap().to_string(),
                    &library["name"].as_str().unwrap().to_string(),
                )
            } else {
                library_downloads["path"].as_str().unwrap().to_string()
            };

            download_file_if_not_exists(&PathBuf::from(full_path), full_url, 0).await;
        }
    }
    for library in version_json["libraries"].as_array().unwrap() {
        let library_downloads = if !library.get("downloads").is_none() {
            &library["downloads"]
        } else {
            &library
        };
        if !library_downloads.get("url").is_none() {
            let url = library["url"].as_str().unwrap();
            let full_url = convert_to_full_url(
                url.to_string(),
                library["name"].as_str().unwrap().to_string(),
            );
            let full_path = convert_to_full_path(
                get_libraries_directory().to_str().unwrap().to_string(),
                &library["name"].as_str().unwrap().to_string(),
            );

            download_file_if_not_exists(&PathBuf::from(full_path), full_url, 0).await;
        }
    }

    fs::remove_dir_all(launcher_dir.join("temp")).unwrap();
}

pub async fn download_fabric(version_loader: &VersionLoader) {
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
    .await;
    let mut child = Command::new(get_java("8".to_string()).await)
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
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            println!("[stderr] {}", line);
        }
    });

    let stdout = child.stdout.take().expect("Failed to open stdout");
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("[java stdout] {}", line);
            }
        }
    });
}

pub async fn get_available_fabric_versions(version_id: &String) -> Vec<String> {
    let mut global_cache = GLOBAL_CACHE.lock().await;
    if global_cache.fabric_mc_versions.is_none() {
        let url = "https://meta.fabricmc.net/v2/versions/game";
        let map: Vec<FabricMinecraftVersion> =
            reqwest::get(url).await.unwrap().json().await.unwrap();
        global_cache.fabric_mc_versions = Some(map);
    }
    if global_cache.fabric_installers.is_none() {
        let url = "https://meta.fabricmc.net/v2/versions/installer";
        let map: Vec<FabricInstaller> = reqwest::get(url).await.unwrap().json().await.unwrap();
        global_cache.fabric_installers = Some(map);
    }
    if global_cache.fabric_loaders.is_none() {
        let url = "https://meta.fabricmc.net/v2/versions/loader";
        let map: Vec<FabricLoader> = reqwest::get(url).await.unwrap().json().await.unwrap();
        global_cache.fabric_loaders = Some(map);
    }

    let map = &global_cache.fabric_mc_versions;
    let unwrapped_map = map.clone().unwrap_or(Vec::new());
    let v = unwrapped_map.iter().find(|x| &x.version == version_id);
    if v.is_none() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let loaders = &global_cache.fabric_loaders;
    for loader in loaders.clone().unwrap() {
        result.push(format!("{}-{}", version_id.to_string(), loader.version));
    }
    result
}
