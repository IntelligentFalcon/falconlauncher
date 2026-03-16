use crate::directory_manager::get_falcon_launcher_directory;
use crate::structs::MinecraftVersion;
use crate::utils::get_downloaded_versions;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::create_dir_all;
use std::path::PathBuf;

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
pub async fn load_config(cfg: &mut Config) {
    let conf = load().await;
    cfg.launch_options = conf.launch_options;
    cfg.launcher_settings = conf.launcher_settings;
}

async fn load() -> Config {
    initialize_configuration_file();
    let content = fs::read_to_string(get_config_directory());
    let config: Config = serde_ini::from_str(content.unwrap().as_str()).unwrap_or(default_config());
    
    config
}
pub fn default_config() -> Config {
    Config {
        launch_options: LaunchOptions {
            username: "".to_string(),
            ram_usage: 2048,
        },
        launcher_settings: LauncherSettings {
            language: "en".to_string(),
        },
        download_settings: DownloadSettings { mirror: "9craft".to_string() },
    }
}
fn initialize_configuration_file() {
    if !get_config_directory().exists() {
        create_dir_all(get_config_directory().parent().unwrap()).unwrap();
        default_config().write_to_file()
    }
}
fn get_config_directory() -> PathBuf {
    get_falcon_launcher_directory().join("launcher-settings.ini")
}
