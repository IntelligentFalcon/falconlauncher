use crate::services::directory_manager::{get_config_directory, get_falcon_launcher_directory};
use crate::models::config::*;
use std::fs;
use std::fs::create_dir_all;
use std::path::PathBuf;
use crate::models::error::Void;

pub fn load_config(cfg: &mut Config) {
    let conf = load();
    cfg.launch_options = conf.launch_options;
    cfg.launcher_settings = conf.launcher_settings;

}
pub fn load() -> Config {
    initialize_configuration_file();
    let content = fs::read_to_string(get_config_directory());
    let config: Config = serde_ini::from_str(content.unwrap().as_str()).unwrap_or(Config::default());
    
    config
}
fn initialize_configuration_file() -> Void{
    if !get_config_directory().exists() {
        create_dir_all(get_config_directory().parent().unwrap()).unwrap();
        return Config::default().write_to_file();
    }
    Ok(())
}
