use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::models::error::AppError;
use crate::models::platform;

#[derive(Debug, Serialize, Deserialize)]
pub struct Java {
    pub path: PathBuf,
    pub version: String,
}

impl Java {
    pub fn new(path: PathBuf) -> Result<Java, AppError> {
        let release = path.join("release");
        let reader = std::fs::read_to_string(release).map_err(|e| AppError::FileReadFailed(e.to_string()))?;
        let line = reader
            .lines()
            .find(|line| line.starts_with("JAVA_VERSION="))
            .ok_or(AppError::InvalidJavaVersion("JAVA_VERSION was not found at RELEASE maybe it doesnt exist?!".to_string()))?;
        let version = line
            .strip_prefix("JAVA_VERSION=")
            .ok_or(AppError::InvalidJavaVersion("JAVA_VERSION was not found at RELEASE maybe it doesnt exist?!".to_string()))?
            .replace("\"", "");

        Ok(Java { path, version })
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
