use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::models::downloader;
use crate::models::downloader::VersionLoader;
use crate::models::platform::get_current_os;
use crate::services::directory_manager::{get_libraries_directory, get_versions_directory};
use crate::services::utils::{extend_once, parse_library_name_to_path};

impl PartialEq for VersionType {
    fn eq(&self, other: &Self) -> bool {
        other == self
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
// #[derive(PartialEq)]
pub enum VersionType {
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
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
    pub fn from_folder(directory: PathBuf) -> MinecraftVersion {
        let ignored_jsons = vec![
            "tlauncheradditional.json",
            "usercache.json",
            "usernamecache.json",
        ];
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
        Self {
            id: name,
            version_path: directory.as_path().to_str().unwrap().to_string(),
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
                            Some(natives) => {
                                let p = if natives.get("path").is_none() {
                                    let url = natives["url"].as_str().unwrap();
                                    let url_https_less =
                                        url.replace("https://", "").replace("http://", "");
                                    let url_args = url_https_less.split("/").collect::<Vec<&str>>();
                                    let path = url_https_less.replace(url_args[0], "");
                                    path
                                } else {
                                    natives.get("path").unwrap().as_str().unwrap().to_string()
                                };
                                let path = libraries_path.join(p).to_str().unwrap().to_string();
                                libraries.push(path.replace("/", "\\"));
                            }
                        }
                    }
                }
                continue;
            }
            let library_info = downloader::library_from_value(library);
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

#[derive(Debug,Serialize, Deserialize)]
pub struct  VersionCategory {
    pub versions: Vec<VersionLoader>,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MinecraftVersion {
    pub id: String,
    pub version_path: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VersionBase {
    VANILLA,
    FORGE,
    NEOFORGE,
    FABRIC,
    LITELOADER,
}