use crate::directory_manager::get_versions_directory;
use crate::structs;
use crate::version_manager::load_version_manifest;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn get_current_os() -> String {
    structs::parse_os(sys_info::os_type().expect("Unsupported Operating System"))
}

pub async fn load_versions() -> Vec<String> {
    let json = load_version_manifest().await;
    let manifest = get_versions_directory()
        .unwrap()
        .join("version_manifest_v2.json");
    if is_connected_to_internet().await && manifest.exists() {
        let mut f = File::open(manifest.as_path()).unwrap();
        let mut text = String::new();
        f.read_to_string(&mut text).unwrap();
        let v: Value = serde_json::from_str(text.as_str()).expect("Unable to parse json");
        let versions = v.get("versions").unwrap().as_array().unwrap();
        let res = versions
            .iter()
            .filter(|ver| ver.get("type").unwrap() == "release")
            .map(|ver| ver.get("id").unwrap().as_str().unwrap().to_string())
            .collect::<Vec<String>>();
        return res;
    }

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
pub async fn is_connected_to_internet() -> bool {
    let req = reqwest::get("https://jsonplaceholder.typicode.com/todos/1");
    match req.await {
        Ok(_) => true,
        Err(_) => false,
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
