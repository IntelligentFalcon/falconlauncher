use crate::models::error::{download_error, json_read_err, request_error, Returns, Void};
use crate::models::java::Java;
use crate::models::mirror::Mirror;
use crate::models::platform;
use crate::models::platform::{get_current_os, get_current_os_with_architecture};
use crate::services::directory_manager::{
    auto_detect_javas, get_java_dir, get_launcher_java_directory,
};
use crate::services::downloader::download_file_if_not_exists;
use crate::services::utils::{is_connected_to_internet, load_json_url};
use serde_json::Value;
use std::fs;
use std::fs::{create_dir, create_dir_all, remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::ZipArchive;
use zip_extract::extract;

pub async fn download_java(id: &String, mirror: &Mirror) {
    let os = platform::get_current_os();
    let mut url = if os == "windows" {
        mirror.parse_url(&format!(
            "https://corretto.aws/downloads/latest/amazon-corretto-{id}-x64-windows-jdk.zip"
        ))
    } else if os == "linux" {
        mirror.parse_url(&format!(
            "https://corretto.aws/downloads/latest/amazon-corretto-{id}-x64-linux-jdk.tar.gz"
        ))
    } else {
        mirror.parse_url(&format!(
            "https://corretto.aws/downloads/latest/amazon-corretto-{id}-x64-macos-jdk.tar.gz"
        ))
    };

    let file_name = url.split("/").last().unwrap_or("");
    let zip_file_path = get_launcher_java_directory().join(file_name);
    let mut output_folder = get_launcher_java_directory().join(id);
    if output_folder.join("bin").exists() {
        return;
    }
    let resp = reqwest::get(&url).await.unwrap();
    let mut file = File::create(&zip_file_path).unwrap();
    file.write(resp.bytes().await.unwrap().as_ref()).unwrap();
    let mut zip_file = File::open(&zip_file_path).unwrap();
    extract(&zip_file, &mut output_folder, false).expect("Extraction of java zip file failed!");
    remove_file(&zip_file_path).expect("TODO: deletion of zip file failed");
    let dirs = output_folder.read_dir().unwrap();
    for dir in dirs {
        let unwrapped_dir = dir.unwrap();
        if unwrapped_dir.file_type().unwrap().is_dir() {
            for entry in unwrapped_dir.path().read_dir().unwrap() {
                let path = entry.unwrap().path();
                let new_path = path
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join(path.file_name().unwrap());
                println!(
                    "Extracting java file: from {} to {}",
                    path.display(),
                    new_path.display()
                );
                fs::rename(path.as_path(), new_path.as_path()).unwrap();
            }
            fs::remove_dir_all(unwrapped_dir.path()).unwrap();
        }
    }
}

pub async fn get_java(java: String, id: String, mirror: &Mirror) -> PathBuf {
    let os = platform::get_current_os();
    if !get_launcher_java_directory().join(&id).exists() {
        let jdk = auto_detect_javas();
        if jdk.is_ok() {
            let jdk_unwrapped = jdk.unwrap();
            let mut filtered = jdk_unwrapped
                .iter()
                .filter(|java| java.get_version_id() == id);
            if filtered.clone().count() > 0 {
                println!("{}", filtered.clone().count());
                return filtered.next().unwrap().path.clone();
            }
        }
    }
    let java = find_or_download_java(&java,&id, mirror).await;
        
    if os == "windows" {
        get_launcher_java_directory()
            .join(&id)
            .join("bin")
            .join("javaw.exe")
    } else {
        get_launcher_java_directory()
            .join(&id)
            .join("bin")
            .join("java")
    }
}

pub async fn find_or_download_java(java: &String, version: &String, mirror: &Mirror) -> Returns<Java> {
    let runtime_dir = get_java_dir().join(&java);
    if is_connected_to_internet().await {
        let url = mirror.parse_url(&"https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json".to_string());
        let current_os = get_current_os_with_architecture();

        let json = load_json_url(&url.to_string()).await;
        if json.is_none() {
            return Err(download_error(
                "Couldn't get or read the runtime json manifest file.".to_string(),
            ));
        }
        let json = json.unwrap();
        let runtime_arr = &json[current_os][java];
        let runtime_v = runtime_arr
            .as_array()
            .unwrap()
            .iter()
            .find(|x| {
                x["version"]["name"]
                    .as_str()
                    .unwrap()
                    .to_lowercase()
                    .starts_with(version.as_str())
            })
            .unwrap_or(&runtime_arr[0]);
        let runtime_manifest_url = mirror.parse_url(&runtime_v["manifest"]["url"].as_str().unwrap().to_string());
        let runtime_manifest = reqwest::get(runtime_manifest_url).await;
        if runtime_manifest.is_err() {
            return Err(download_error("Couldn't get runtime manifest.".to_string()));
        }
        let runtime_manifest = runtime_manifest.unwrap().json().await;
        if runtime_manifest.is_err() {
            return Err(download_error(
                "Failed to read the runtime json file.".to_string(),
            ));
        }
        let runtime_manifest: Value = runtime_manifest.unwrap();
        let files = &runtime_manifest["files"];
        for (k, v) in files.as_object().unwrap() {
            let file_type = v["type"].as_str().unwrap();
            if file_type == "file" {
                let download_raw = &v["downloads"]["raw"];
                let url = download_raw["url"].as_str().unwrap();
                let size = download_raw["size"].as_u64().unwrap();

                create_dir_all(&runtime_dir.join(k));
                download_file_if_not_exists(&runtime_dir.join(k), url.to_string(), size);
            } else {
                create_dir(runtime_dir.join(k)).unwrap();
            }
        }
    }

    Ok(Java::new(runtime_dir))
}
