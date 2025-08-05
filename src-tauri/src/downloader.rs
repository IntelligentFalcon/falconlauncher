use crate::directory_manager::{
    get_assets_directory, get_libraries_directory, get_natives_folder, get_version_directory,
    get_versions_directory,
};
use crate::game_launcher::{update_download, update_download_bar, update_download_status};
use crate::structs::{library_from_value, LibraryRules, MinecraftVersion};
use crate::utils::{get_current_os, verify_file_existence};
use crate::version_manager::load_version_manifest;
use serde_json::Value;
use std::fs;
use std::fs::{create_dir_all, exists, File};
use std::io::Write;
use std::ops::Index;
use std::path::PathBuf;
use tauri::async_runtime::block_on;
use tauri::AppHandle;
use zip_extract::extract;

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
        &PathBuf::from(asset_index_path),
        url.to_string(),
        total_size,
    )
    .await;
    let content = reqwest::get(url)
        .await
        .expect("Failed to download file.")
        .text()
        .await
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

    println!("Downloading version {} process started.", id);
    let manifest = load_version_manifest().await;
    if !version.is_installed() {
        match manifest {
            None => {}
            Some(val) => {
                download_from_manifest(id, val).await;
            }
        }
    }
    let content = fs::read_to_string(PathBuf::from(version.get_json())).unwrap();

    let json: Value = serde_json::from_str(&content).unwrap();

    download_libraries(&json["libraries"], &id, app_handle).await;
    if !json.get("downloads").is_none() {
        download_client(&json["downloads"]["client"], &id).await;
    }

    if json.get("assetIndex") != None {
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
                    if reqwest::get(url.clone()).await.unwrap().status().is_success() {
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
            let path = val["path"].as_str().unwrap();
            let full_path = get_libraries_directory().join(path);
            let size = val["size"].as_u64().unwrap();
            let url = val["url"].as_str().unwrap();
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

async fn download_file(url: String, dest: String) {
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
