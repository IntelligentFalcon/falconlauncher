use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::models::platform;
use crate::services::directory_manager::get_launcher_java_directory;

#[derive(Debug, Serialize, Deserialize)]
pub struct Java {
    pub(crate) path: PathBuf,
    version: String,
}
impl Java {
    pub fn new(path: PathBuf) -> Java {
        let release = path.join("release");
        let reader = std::fs::read_to_string(release).unwrap();
        let line = reader
            .lines()
            .find(|line| line.starts_with("JAVA_VERSION="))
            .unwrap();
        let version = line
            .strip_prefix("JAVA_VERSION=")
            .unwrap()
            .replace("\"", "");

        Java { path, version }
    }
    pub fn get_bin_file(&self) -> PathBuf {
        let os = platform::get_current_os();

        if os == "windows" {
            self.path
                .join("bin")
                .join("javaw.exe")
        } else {
                self.path
                .join("bin")
                .join("java")
        }
    }
    pub fn get_version_id(&self) -> String {
        self.version.split(".").next().unwrap().to_string()
    }
    
}
