use crate::services::directory_manager::{get_config_directory, get_falcon_launcher_directory};
use crate::models::config::*;
use std::fs;
use std::fs::create_dir_all;
use std::path::PathBuf;

pub fn load_config(cfg: &mut Config) {
    let conf = load();
    cfg.launch_options = conf.launch_options;
    cfg.launcher_settings = conf.launcher_settings;

}
pub fn load() -> Config {
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
