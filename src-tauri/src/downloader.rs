use crate::directory_manager::{
    get_assets_directory, get_libraries_directory, get_minecraft_directory, get_natives_folder,
    get_version_directory, get_versions_directory,
};
use crate::structs::OperatingSystem::{Linux, Windows};
use crate::structs::{library_from_value, LibraryRules, OperatingSystem};
use crate::utils;
use crate::utils::{get_current_os, load_json_url, verify_file_existence};
use crate::version_manager::load_version_manifest;
use serde_json::{Map, Value};
use std::fmt::format;
use std::fs::{create_dir_all, exists, File};
use std::io::Write;
use std::path::PathBuf;
use tauri::async_runtime::{block_on, spawn};
use tauri::{AppHandle, Emitter};
use zip_extract::extract;

async fn download_assets(value: &Value) {
    let id = value["id"].as_str().unwrap();
    let url = value["url"].as_str().unwrap();
    let total_size = value["totalSize"].as_u64().unwrap();
    let size = value["size"].as_u64().unwrap();
    let mut json: Option<Value> = None;
    let asset_index_path = get_assets_directory()
        .expect("Couldn't get minecraft directory")
        .join("indexes")
        .join(format!("{id}.json"))
        .to_str()
        .unwrap()
        .to_string();
    if !verify_file_existence(&asset_index_path, size) {
        download_file(url.to_string(), asset_index_path).await;
    }
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
                    .unwrap()
                    .join("objects")
                    .join(id.as_str())
                    .join(hash);
                if !verify_file_existence(&path.to_str().unwrap().to_string(), size) {
                    download_file(url, path.to_str().unwrap().to_string()).await;
                }
            }
        }
        None => {}
    }
}

pub async fn download_version(id: String, app_handle: &AppHandle) {
    println!("Downloading version {} process started.", id);
    let manifest = load_version_manifest().await;
    println!("Loaded version manifest");
    match manifest {
        None => {}
        Some(val) => {
            let version = val["versions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["id"].as_str().unwrap() == id)
                .expect("Couldn't find version in manifest.");
            let version_url = version["url"].as_str().unwrap();

            download_file(
                version_url.to_string(),
                get_version_directory(&id)
                    .unwrap()
                    .join(format!("{}.json", id))
                    .to_str()
                    .unwrap()
                    .to_string(),
            )
            .await;
            app_handle.emit("progressBar", 10).unwrap();

            let json = load_json_url(&version_url.to_string())
                .await
                .expect("Couldn't find the version");
            download_assets(&json["assetIndex"]).await;
            app_handle.emit("progressBar", 50).unwrap();
            println!("Downloaded assets");
            download_libraries(&json["libraries"], &id).await;
            app_handle.emit("progressBar", 80).unwrap();
            println!("Downloaded libraries");
            download_client(&json["downloads"]["client"], &id).await;
            println!("Downloaded client!");
            app_handle.emit("progressBar", 90).unwrap();
        }
    }
}
async fn download_client(value: &Value, version: &String) {
    let size = value["size"].as_u64().unwrap();
    let url = value["url"].as_str().unwrap();
    let path = get_versions_directory()
        .unwrap()
        .join(&version)
        .join(format!("{}.jar", version));
    if !verify_file_existence(&path.to_str().unwrap().to_string(), size) {
        download_file(url.to_string(), path.to_str().unwrap().to_string()).await;
    }
}

async fn download_libraries(libraries: &Value, version: &String) {
    let libraries_path = get_libraries_directory().expect("Getting library path failed");
    for library in libraries.as_array().unwrap() {
        if library.get("downloads").unwrap().get("artifact").is_none() {
            download_classifiers(
                library.get("downloads").unwrap().get("classifiers"),
                version,
            )
            .await;
            continue;
        }
        let library_info = library_from_value(library);
        let os = get_current_os();
        let rules = fetch_rules(library.get("rules"));
        download_classifiers(
            library.get("downloads").unwrap().get("classifiers"),
            version,
        )
        .await;
        if rules.allowed_oses.contains(&os) && !rules.disallowed_oses.contains(&os) {
            let path = libraries_path.join(&library_info.path.as_str());
            if !verify_file_existence(&path.to_str().unwrap().to_string(), library_info.size) {
                download_file(
                    library_info.url.as_str().to_string(),
                    path.to_str().unwrap().to_string(),
                )
                .await;
            }
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
            let full_path = get_libraries_directory().unwrap().join(path);
            let size = val["size"].as_u64().unwrap();
            let url = val["url"].as_str().unwrap();
            if !verify_file_existence(&full_path.to_str().unwrap().to_string(), size) {
                download_file(url.to_string(), full_path.to_str().unwrap().to_string()).await;
            }
            let file = File::open(full_path.to_str().unwrap().to_string());
            let natives_path = get_natives_folder(version).unwrap();
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
