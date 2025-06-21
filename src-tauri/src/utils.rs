use crate::directory_manager::get_versions_directory;
use crate::structs;
use crate::structs::MinecraftVersion;
use crate::version_manager::load_version_manifest;
use serde_json::Value;
use std::ffi::OsStr;
use std::fs::{read_dir, File};
use std::io::Read;
use std::path::Path;
use tauri::http::Response;

pub fn get_current_os() -> String {
    structs::parse_os(sys_info::os_type().expect("Unsupported Operating System"))
}
fn load_downloaded_versions() {
    let dir = get_versions_directory();
    for folder in dir
        .read_dir()
        .unwrap()
        .map(|x| x.unwrap())
        .filter(|x| x.metadata().unwrap().is_dir())
    {
        folder
            .path()
            .read_dir()
            .unwrap()
            .map(|x| x.unwrap())
            .filter(|x| x.path().extension().unwrap() == "json")
            .for_each(|x| {})
    }
}
pub async fn load_versions() -> Vec<MinecraftVersion> {
    let mut versions = Vec::new();
    versions = get_versions_directory()
        .read_dir()
        .unwrap()
        .map(|x| x.unwrap())
        .filter(|x| {
            if x.path().is_file() {
                return false;
            }
            let children_files = x.path().read_dir().unwrap();
            return children_files
                .map(|f| f.unwrap())
                .filter(|f| f.path().is_file() && f.path().extension().unwrap() == "json")
                .count()
                > 0;
        })
        .map(|v| {
            MinecraftVersion::from_folder(
                get_versions_directory().join(v.file_name().to_str().unwrap().to_string()),
            )
            .unwrap()
        })
        .collect();
    if is_connected_to_internet().await {
        let json = load_version_manifest().await;

        let founded_versions = match json {
            None => Vec::new(),
            Some(v) => {
                let versions = v.get("versions").unwrap().as_array().unwrap();
                versions
                    .iter()
                    .filter(|ver| ver.get("type").unwrap() == "release")
                    .map(|ver| {
                        MinecraftVersion::from_id(
                            ver.get("id").unwrap().as_str().unwrap().to_string(),
                        )
                    })
                    .collect()
            }
        };
        versions.extend(founded_versions);
    }
    versions
}

// pub fn get_local_versions() -> Vec<String> {
//
// }
/// Verifies if file exists and is not broken by the expected file size
pub fn verify_file_existence(path_str: &String, expected_size: u64) -> bool {
    let path = Path::new(&path_str);
    if !path.exists() {
        false
    } else {
        let file = File::open(path).unwrap();
        let metadata = file.metadata().unwrap();
        metadata.len() == expected_size
    }
}
pub async fn is_connected_to_internet() -> bool {
    let req = reqwest::get("https://jsonplaceholder.typicode.com/todos/1");
    match req.await {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub async fn load_json_url(url: &String) -> Option<Value> {
    let result = reqwest::get(url).await.unwrap();
    let text = result.text().await.unwrap_or(String::new());
    Some(serde_json::from_str(text.as_str()).expect("JSON File isn't well formatted."))
}

pub fn vec_to_string(vec: Vec<String>, separator: String) -> String {
    let mut builder = "".to_string();
    for s in vec {
        builder.push_str(&s);
        builder.push_str(&separator);
    }
    builder.remove(builder.len() - 1);
    builder
}
