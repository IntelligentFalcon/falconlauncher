use crate::directory_manager::get_versions_directory;
use crate::version_manager::VersionInfo;
use serde_json::Value;
use std::fs;
use std::fs::exists;
use std::path::{Path, PathBuf};

pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub total_size: u64,
    pub url: String,
}

pub struct LibraryInfo {
    pub name: String,
    pub size: u64,
    pub path: String,
    pub url: String,
}
pub fn library_from_value(value: &Value) -> LibraryInfo {
    let library_name = value
        .get("name")
        .expect("Parsing library_name failed")
        .as_str()
        .expect("Parsing library_name failed");
    let library_downloads = value.get("downloads").unwrap();
    let library_artifact = library_downloads
        .get("artifact")
        .expect("Parsing library_downloads failed");

    let library_path = library_artifact
        .get("path")
        .expect("Parsing library path failed")
        .as_str()
        .expect("Parsing library path failed");
    let library_url = library_artifact
        .get("url")
        .expect("Parsing library_url failed")
        .as_str();
    let library_size = library_artifact
        .get("size")
        .expect("Parsing library_size failed")
        .as_u64()
        .expect("Parsing library_size failed");
    LibraryInfo {
        name: library_name.to_string(),
        size: library_size,
        path: library_path.to_string(),
        url: library_url.unwrap().to_string(),
    }
}
pub enum OperatingSystem {
    Windows,
    Linux,
    MacOS,
}

pub fn parse_os(os: String) -> String {
    os.to_lowercase().replace("darwin", "osx")
}
pub struct LibraryRules {
    pub allowed_oses: Vec<String>,
    pub disallowed_oses: Vec<String>,
}

pub struct MinecraftVersion {
    pub id: String,
    pub version_path: String,
}

impl MinecraftVersion {
    pub fn is_installed(&self) -> bool {
        exists(&self.version_path).unwrap()
    }

    pub fn new(id: String, version_folder: String) -> Self {
        let versions_dir = get_versions_directory();
        Self {
            id,
            version_path: versions_dir
                .join(version_folder)
                .to_str()
                .unwrap()
                .to_string(),
        }
    }
    pub fn from_id(id: String) -> Self {
        MinecraftVersion::new(id.clone(), id)
    }
    pub fn from_folder(directory: PathBuf) -> Option<Self> {
        let mut filtered_file: Vec<PathBuf> = directory
            .read_dir()
            .unwrap()
            .map(|x| x.unwrap().path())
            .filter(|x| x.is_file() && x.extension().unwrap() == "json")
            .collect();
        let count = &filtered_file.len();
        if count == &1 {
            let next = &filtered_file[0];
            let json: Value =
                serde_json::from_str(fs::read_to_string(next).unwrap().as_str()).unwrap();
            let name = json["id"].as_str().unwrap().to_string();
            return Some(Self {
                id: name,
                version_path: directory.as_path().to_str().unwrap().to_string(),
            });
        }
        None
    }
}
