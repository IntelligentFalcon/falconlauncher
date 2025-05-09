use crate::downloader::load_version_manifest;
use crate::structs;
use serde_json::Value;
pub fn get_current_os() -> String {
    structs::parse_os(sys_info::os_type().expect("Unsupported Operating System"))
}

pub fn load_versions() -> Vec<String> {
    let json = load_version_manifest();
    match json {
        None => Vec::new(),
        Some(v) => {
            let mut versions = v.get("versions").unwrap().as_array().unwrap();
            versions
                .iter()
                .filter(|ver| ver.get("type").unwrap() == "release")
                .map(|ver| ver.get("id").unwrap().as_str().unwrap().to_string())
                .collect()
        }
    }
}
