use crate::directory_manager::get_versions_directory;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

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
#[derive(Clone, PartialEq)]
pub struct MinecraftVersion {
    pub id: String,
    pub version_path: String,
}

impl MinecraftVersion {
    pub fn is_installed(&self) -> bool {
        PathBuf::from(self.get_json()).exists()
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
    pub fn get_json(&self) -> String {
        self.version_path.clone() + "/" + self.id.clone().as_str() + ".json"
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
    pub fn is_forge(&self) -> bool {
        self.id.contains("forge")
    }
    pub fn load_json(&self) -> Value {
        if !self.is_installed() {
            Value::from("")
        } else {
            let content = fs::read_to_string(PathBuf::from(self.get_json())).unwrap();
            serde_json::from_str(&content).unwrap()
        }
    }
    pub fn get_inherited(&self) -> MinecraftVersion {
        let json = self.load_json();
        if json.get("inheritsFrom").is_none() {
            self.clone()
        } else {
            let inherited = json["inheritsFrom"].as_str().unwrap().to_string();
            let version = MinecraftVersion::from_id(inherited);
            version
        }
    }
    pub fn is_fabric(&self) -> bool {
        self.id.contains("fabric")
    }
}

pub struct ModInfo {
    pub path: String,
    pub mod_id: String,
    pub display_name: String,
    pub version: String,
}
impl ModInfo {
    pub fn new(path: String, mod_id: String, display_name: String, version: String) -> Self {
        Self {
            path,
            mod_id,
            display_name,
            version,
        }
    }
}
