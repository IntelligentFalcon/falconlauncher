use crate::directory_manager::get_falcon_launcher_directory;
use ini::Ini;
use std::fs::{create_dir_all, exists, File};
use std::io::{BufReader, Read};
use std::path::PathBuf;

pub struct Config {
    pub username: String,
    pub ram_usage: u64,
    pub java_path: String,
}
pub fn dump(config: &Config) {
    let mut conf = Ini::new();
    conf.with_section(Some("LaunchOptions"))
        .set("ram_usage", &config.ram_usage.to_string())
        .set("username", &config.username);
    conf.write_to_file(get_config_directory()).unwrap()
}
fn get_ini() -> Ini {
    let file = File::open(get_config_directory()).unwrap();
    Ini::read_from(&mut BufReader::new(file)).expect("Reading failed!")
}
pub fn load_config(config: &mut Config) {
    let conf = load();
    config.username = conf.username;
    config.ram_usage = conf.ram_usage;
}
fn load() -> Config {
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
    Config {
        username,
        ram_usage,
        java_path: "java".to_string(),
    }
}
fn default_config() -> Ini {
    let mut conf = Ini::new();
    conf.with_section(Some("LaunchOptions"))
        .set("ram_usage", "2048")
        .set("username", "Steve");
    conf
}
fn initialize_configuration_file() {
    if !exists(get_config_directory()).unwrap() {
        create_dir_all(get_config_directory().parent().unwrap()).unwrap();
        default_config()
            .write_to_file(get_config_directory())
            .expect("Writing ini file failed");
    }
}
fn get_config_directory() -> PathBuf {
    get_falcon_launcher_directory()
        .join("launcher-settings.ini")
}
