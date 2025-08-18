use crate::directory_manager::{get_libraries_directory, get_versions_directory};
use crate::structs::VersionBase::{FABRIC, FORGE, LITELOADER, NEOFORGE, VANILLA};
use crate::utils::{extend_once, get_current_os, parse_library_name_to_path};
use serde::{Deserialize, Serialize};
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
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MinecraftVersion {
    pub id: String,
    pub version_path: String,
    pub base: VersionBase,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum VersionBase {
    VANILLA,
    FORGE,
    NEOFORGE,
    FABRIC,
    LITELOADER,
}
struct Library {
    pub name: String,
    pub path: String,
    pub url: Option<String>,
}
impl MinecraftVersion {
    pub fn is_installed(&self) -> bool {
        PathBuf::from(self.get_json()).exists()
    }

    pub fn new(id: String, version_folder: String) -> Self {
        let versions_dir = get_versions_directory();
        let base = if version_folder.to_lowercase().contains("forge") {
            FORGE
        } else if version_folder.to_lowercase().contains("fabric") {
            FABRIC
        } else if version_folder.to_lowercase().contains("liteloader") {
            LITELOADER
        } else if version_folder.to_lowercase().contains("neoforge") {
            NEOFORGE
        } else {
            VANILLA
        };
        Self {
            id,
            version_path: versions_dir
                .join(version_folder)
                .to_str()
                .unwrap()
                .to_string(),
            base,
        }
    }
    pub fn get_json(&self) -> String {
        self.version_path.clone() + "/" + self.id.clone().as_str() + ".json"
    }
    pub fn from_id(id: String) -> Self {
        MinecraftVersion::new(id.clone(), id)
    }
    pub fn from_folder(directory: PathBuf) -> MinecraftVersion {
        let ignored_jsons = vec!["tlauncheradditional.json"];
        let file: PathBuf = directory
            .read_dir()
            .unwrap()
            .map(|x| x.unwrap().path())
            .find(|x| {
                x.is_file()
                    && (x.extension().unwrap() == "json"
                        && !ignored_jsons.contains(
                            &x.file_name()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_lowercase()
                                .as_str(),
                        ))
            })
            .unwrap();
        let json: Value = serde_json::from_str(fs::read_to_string(file).unwrap().as_str()).unwrap();
        let name = json["id"].as_str().unwrap().to_string();
        let version_folder = directory.to_str().unwrap().to_string();
        let base = if version_folder.to_lowercase().contains("forge") {
            FORGE
        } else if version_folder.to_lowercase().contains("fabric") {
            FABRIC
        } else if version_folder.to_lowercase().contains("liteloader") {
            LITELOADER
        } else if version_folder.to_lowercase().contains("neoforge") {
            NEOFORGE
        } else {
            VANILLA
        };
        Self {
            id: name,
            version_path: directory.as_path().to_str().unwrap().to_string(),
            base
        }
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
    fn get_library_paths(&self) -> Vec<String> {
        let value = &self.load_json()["libraries"];
        let libraries_path = get_libraries_directory();
        let mut libraries = vec![];

        for library in value.as_array().unwrap() {
            if library.get("downloads").is_none() {
                let library_name = library["name"].as_str().unwrap();
                let library_path_str =
                    parse_library_name_to_path(library_name.to_string()).replace("/", "\\");
                let library_path = PathBuf::from(&library_path_str);
                if library_path.exists() && !libraries.contains(&library_path_str) {
                    libraries.push(library_path_str);
                }
                continue;
            } else if library["downloads"].get("artifact").is_none() {
                let classifiers = &library["downloads"].get("classifiers");
                let os = get_current_os();
                match classifiers {
                    None => {}
                    Some(val) => {
                        let natives = val.get(format!("natives-{os}"));
                        match natives {
                            None => {}
                            Some(_) => {
                                let path = libraries_path
                                    .join(natives.unwrap().get("path").unwrap().as_str().unwrap())
                                    .to_str()
                                    .unwrap()
                                    .to_string();
                                libraries.push(path.replace("/", "\\"));
                            }
                        }
                    }
                }
                continue;
            }
            let library_info = library_from_value(library);
            let os = get_current_os();
            let path = libraries_path
                .join(&library_info.path.as_str().replace("/", "\\"))
                .to_str()
                .unwrap()
                .to_string()
                .to_string();

            if !libraries.contains(&path) {
                libraries.push(path);
            }
        }

        libraries
    }
    pub fn get_libraries(&self) -> Vec<String> {
        let mut libraries = self.get_library_paths();
        let libraries_2 = self.get_inherited().get_library_paths();
        libraries = libraries
            .into_iter()
            .filter(|x| {
                let path = PathBuf::from(x);
                let parent = path.parent().unwrap();
                let artifact = parent
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap();

                !libraries_2
                    .iter()
                    .map(|x| {
                        PathBuf::from(x)
                            .parent()
                            .unwrap()
                            .parent()
                            .unwrap()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_lowercase()
                            .to_string()
                    })
                    .collect::<Vec<String>>()
                    .contains(&artifact.to_string())
            })
            .collect::<Vec<String>>();
        libraries = extend_once(libraries, libraries_2);
        libraries
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
#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub online: bool,
    pub uuid: uuid::Uuid,
}

#[derive(Serialize)]
pub struct VersionCategory {
    pub versions: Vec<MinecraftVersion>,
    pub name: String,
}
