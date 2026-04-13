use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub fn get_version_id(&self) -> String {
        self.version.split(".").next().unwrap().to_string()
    }
    
}
