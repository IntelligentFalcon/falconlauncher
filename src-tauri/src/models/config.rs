use std::fs;
use serde::{Deserialize, Serialize};
use crate::services::directory_manager::get_config_directory;

#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchOptions {
    pub username: String,
    pub ram_usage: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LauncherSettings {
    pub language: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadSettings {
    pub mirror: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(rename = "LaunchOptions")]
    pub launch_options: LaunchOptions,
    #[serde(rename = "LauncherSettings")]
    pub launcher_settings: LauncherSettings,
    #[serde(rename = "DownloadSettings")]
    pub download_settings: DownloadSettings
}

impl Config {
    pub fn write_to_file(&self) {
        let text = serde_ini::to_string(self).unwrap();
        fs::write(get_config_directory(), text).unwrap();
    }
}