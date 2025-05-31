use crate::directory_manager::get_falcon_launcher_directory;
use std::fs;
use std::fs::{create_dir_all, exists, read_to_string, File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;
use tauri::utils::acl::Error::WriteFile;
use yaml_rust2::yaml::YamlDecoder;
use yaml_rust2::{Yaml, YamlEmitter, YamlLoader};

pub struct Config {
    pub username: String,
    pub ram_usage: u64,
}
pub fn dump(config: &Config) {
    let yaml = Yaml::from_str(
        format!(
            "
username: {}
ram_usage: {}
  ",
            config.username, config.ram_usage
        )
        .as_str(),
    );
    let mut file = OpenOptions::new()
        .write(true)
        .open(get_config_directory())
        .unwrap();
    let mut out = String::new();
    let mut dumper = YamlEmitter::new(&mut out);
    dumper.dump(&yaml).expect("Dumping failed!");
    file.write(out.as_bytes()).expect(
        format!(
            "Writing on file failed. {}",
            get_config_directory().to_string_lossy()
        )
        .as_str(),
    );
}
fn get_yaml() -> Yaml {
    YamlLoader::load_from_str(
        read_to_string(get_config_directory().as_path())
            .unwrap()
            .as_str(),
    )
    .unwrap()[0]
        .clone()
}
pub fn load() -> Config {
    initialize_configuration_file();
    let yaml = get_yaml();
    let username = yaml["username"].as_str().unwrap().to_string();
    let ram_usage = yaml["ram_usage"].as_i64().unwrap() as u64;
    Config {
        username,
        ram_usage,
    }
}
fn default_config() -> String {
    "
ram_usage: 2048
username: \"Steve\"\

    "
    .to_string()
}
fn initialize_configuration_file() {
    if !exists(get_config_directory()).unwrap() {
        create_dir_all(get_config_directory().parent().unwrap()).unwrap();
        let mut file = File::create(get_config_directory()).unwrap();
        file.write(default_config().as_bytes()).unwrap();
    }
}
fn get_config_directory() -> PathBuf {
    get_falcon_launcher_directory()
        .unwrap()
        .join("launcher-settings.yml")
}
