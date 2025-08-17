use crate::directory_manager::get_falcon_launcher_directory;
use crate::structs::MinecraftVersion;
use crate::utils::{get_downloaded_versions, load_versions};
use ini::Ini;
use std::fs::{create_dir_all, exists, File};
use std::io::{BufReader, Read};
use std::path::PathBuf;
use tauri::async_runtime::block_on;

pub struct Config {
    pub username: String,
    pub ram_usage: u64,
    pub versions: Vec<MinecraftVersion>,
    pub language: String,
}

pub fn dump(config: &Config) {
    let mut conf = Ini::new();
    conf.with_section(Some("LaunchOptions"))
        .set("ram_usage", &config.ram_usage.to_string())
        .set("username", &config.username);
    conf.with_section(Some("LauncherSettings"))
        .set("language", &config.language);
    conf.write_to_file(get_config_directory()).unwrap()
}
fn get_ini() -> Ini {
    let file = File::open(get_config_directory()).expect("Failed to get the ini file.");
    Ini::read_from(&mut BufReader::new(file)).expect("Reading failed!")
}
pub async fn load_config(config: &mut Config) {
    let conf = load().await;
    config.username = conf.username;
    config.ram_usage = conf.ram_usage;
    config.versions = conf.versions;
    config.language = conf.language;
}
async fn load() -> Config {
    initialize_configuration_file();
    let mut conf = get_ini();
    let username = conf
        .with_section(Some("LaunchOptions"))
        .get("username")
        .expect("Could not find username")
        .to_string();
    let ram_usage = conf
        .with_section(Some("LaunchOptions"))
        .get("ram_usage")
        .expect("Could not find ram usage")
        .parse::<u64>()
        .unwrap();
    let language = conf
        .with_section(Some("LauncherSettings"))
        .get("language")
        .unwrap_or("en")
        .parse::<String>()
        .expect("Could not parse language");
    let mut versions = Vec::<MinecraftVersion>::new();
    versions = get_downloaded_versions();
    Config {
        username,
        ram_usage,
        versions,
        language,
    }
}
fn default_config() -> Ini {
    let mut conf = Ini::new();
    conf.with_section(Some("LaunchOptions"))
        .set("ram_usage", "2048")
        .set("username", "Steve");
    conf.with_section(Some("LauncherSettings"))
        .set("language", "en");

    conf
}
fn initialize_configuration_file() {
    if !get_config_directory().exists() {
        create_dir_all(get_config_directory().parent().unwrap()).unwrap();
        default_config()
            .write_to_file(get_config_directory())
            .expect("Writing ini file failed");
    }
}
fn get_config_directory() -> PathBuf {
    get_falcon_launcher_directory().join("launcher-settings.ini")
}
