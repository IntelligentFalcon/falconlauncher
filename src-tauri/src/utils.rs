use crate::structs;
use crate::version_manager::load_version_manifest;
use serde_json::Value;
use std::fs::File;
use std::path::{Path, PathBuf};
use tauri::async_runtime::block_on;
use tauri::command;
use tauri::CursorIcon::Copy;

pub fn get_current_os() -> String {
    structs::parse_os(sys_info::os_type().expect("Unsupported Operating System"))
}

pub async fn load_versions() -> Vec<String> {
    let json = load_version_manifest().await;
    match json {
        None => Vec::new(),
        Some(v) => {
            let versions = v.get("versions").unwrap().as_array().unwrap();
            versions
                .iter()
                .filter(|ver| ver.get("type").unwrap() == "release")
                .map(|ver| ver.get("id").unwrap().as_str().unwrap().to_string())
                .collect()
        }
    }
}
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
pub async fn load_json_url(url: &String) -> Option<Value> {
    let result = reqwest::get(url).await.expect("Failed to download file.");
    let text = result.text().await.expect("Failed to read file.");
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
