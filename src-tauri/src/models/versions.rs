use crate::models::downloader;
use crate::models::downloader::{MinecraftManifestVersion, VersionLoader};
use crate::models::platform::get_current_os;
use crate::services::directory_manager::{get_libraries_directory, get_versions_directory};
use crate::services::utils::{extend_once, parse_library_name_to_path};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{PathBuf, MAIN_SEPARATOR_STR};
use log::debug;
use crate::models::error::AppError;
use crate::models::logger::info;

impl PartialEq for VersionType {
    fn eq(&self, other: &Self) -> bool {
        other == self
    }
}




#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
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
                .to_string_lossy()
                .into_owned(),
        }
    }

    pub fn get_json(&self) -> String {
        format!("{}/{}.json", self.version_path, self.id)
    }

    pub fn from_id(id: String) -> Self {
        MinecraftVersion::new(id.clone(), id)
    }

    pub fn from_folder(directory: PathBuf) -> Result<MinecraftVersion, AppError> {
        let mut target_file = None;

        if let Ok(entries) = directory.read_dir() {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "json" {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if serde_json::from_str::<MinecraftManifestVersion>(&content).is_ok() {
                                    target_file = Some(path);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        let file = target_file.ok_or_else(|| AppError::DirNotFound("Directory not found".to_string()))?;

        let content = fs::read_to_string(&file)
            .map_err(|_| AppError::UnknownError("File read error".to_string()))?;

        let json: MinecraftManifestVersion = serde_json::from_str(&content)
            .map_err(|_| AppError::JsonParseFailed("Parsing json failed".to_string()))?;

        Ok(Self {
            id: json.id,
            version_path: directory.to_string_lossy().into_owned(),
        })
    }

    pub fn is_forge(&self) -> bool {
        self.id.contains("forge")
    }

    pub fn load_json(&self) -> Value {
        if !self.is_installed() {
            Value::String("".to_string())
        } else {
            fs::read_to_string(PathBuf::from(self.get_json()))
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or(Value::Null)
        }
    }

    pub fn get_inherited(&self) -> MinecraftVersion {
        let json = self.load_json();
        if json.get("inheritsFrom").is_none() || json["inheritsFrom"].is_null() {
            if let Some(id) = json.get("id").and_then(|i| i.as_str()) {
                if id.to_lowercase().contains("forge") {
                    if let Some(first_part) = id.split('-').next() {
                        if first_part != "forge" {
                            return MinecraftVersion::from_id(first_part.to_string());
                        }
                    }
                }
            }
            self.clone()
        } else {
            if let Some(inherited) = json["inheritsFrom"].as_str() {
                MinecraftVersion::from_id(inherited.to_string())
            } else {
                self.clone()
            }
        }
    }

    pub fn is_fabric(&self) -> bool {
        self.id.contains("fabric")
    }

    fn get_library_paths(&self) -> Vec<String> {
        let value = &self.load_json()["libraries"];
        let libraries_path = get_libraries_directory();
        let mut libraries = vec![];

        let Some(library_array) = value.as_array() else {
            return libraries;
        };

        for library in library_array {
            if library.get("downloads").is_none() || library["downloads"].is_null() {
                if let Some(library_name) = library.get("name").and_then(|n| n.as_str()) {
                    if let Ok(mut library_path_str) = parse_library_name_to_path(library_name.to_string()) {
                        library_path_str = library_path_str.replace("/", MAIN_SEPARATOR_STR);
                        let library_path = PathBuf::from(&library_path_str);

                        if library_path.exists() && !libraries.contains(&library_path_str) {
                            libraries.push(library_path_str);
                        }
                    }
                }
                continue;
            } else if library["downloads"].get("artifact").is_none() || library["downloads"]["artifact"].is_null() {
                if let Some(classifiers) = library["downloads"].get("classifiers") {
                    let os = get_current_os();
                    if let Some(natives) = classifiers.get(format!("natives-{os}")) {
                        let p = if natives.get("path").is_none() || natives["path"].is_null() {
                            if let Some(url) = natives.get("url").and_then(|u| u.as_str()) {
                                let url_https_less = url.replace("https://", "").replace("http://", "");
                                let mut url_args = url_https_less.split('/');
                                if let Some(first_arg) = url_args.next() {
                                    url_https_less.replacen(first_arg, "", 1)
                                } else {
                                    "".to_string()
                                }
                            } else {
                                "".to_string()
                            }
                        } else {
                            natives["path"].as_str().unwrap_or("").to_string()
                        };

                        if !p.is_empty() {
                            let path = libraries_path.join(p).to_string_lossy().into_owned();
                            let formatted_path = path.replace("/", MAIN_SEPARATOR_STR);
                            if !libraries.contains(&formatted_path) {
                                libraries.push(formatted_path);
                            }
                        }
                    }
                }
                continue;
            }

            // Assuming library_from_value_legacy accepts a JSON Value directly
            let library_info = downloader::library_from_value_legacy(library);
            let os = get_current_os();

            let path = libraries_path
                .join(&library_info.path.replace("\\", MAIN_SEPARATOR_STR))
                .to_string_lossy()
                .into_owned()
                .replace("\\", MAIN_SEPARATOR_STR);

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

                let artifact = path
                    .parent()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                !libraries_2.iter().any(|lib2| {
                    PathBuf::from(lib2)
                        .parent()
                        .and_then(|p| p.parent())
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .map(|n| n.to_lowercase())
                        .unwrap_or_default() == artifact.to_lowercase()
                })
            })
            .collect::<Vec<String>>();

        libraries = extend_once(libraries, libraries_2);
        libraries
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionCategory {
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

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct VersionNameBase {
    pub name: String,
    pub base: String
}