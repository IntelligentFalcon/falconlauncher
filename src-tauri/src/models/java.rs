use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::models::platform;

#[derive(Debug, Serialize, Deserialize)]
pub struct Java {
    pub path: PathBuf,
    pub version: String,
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
                .join("java.exe")
        } else {
                self.path
                .join("bin")
                .join("java")
        }
    }
    
}
