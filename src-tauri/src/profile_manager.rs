use crate::directory_manager::get_profiles_file;
use crate::structs::Profile;
use serde_json::json;
use std::fs;
use std::fs::{read_to_string, File};
use toml::Value;
use uuid::Uuid;

pub fn create_new_profile(username: String, online: bool) {
    let mut profiles: Vec<Profile> = get_profiles();
    if !get_profiles_file().exists() {
        File::create(&get_profiles_file()).expect("Failed to create the file!");
    }
    let uid = Uuid::new_v4();

    profiles.push(Profile {
        name: username,
        online,
        uuid: uid,
    });

    let json_string = serde_json::to_string_pretty(&profiles).unwrap();
    fs::write(get_profiles_file(), json_string).expect("Failed to write the file!");
}

pub fn get_profiles() -> Vec<Profile> {
    serde_json::from_str(
        read_to_string(get_profiles_file())
            .unwrap_or("".to_string())
            .as_str(),
    )
    .unwrap_or(Vec::new())
}

pub fn get_profile(username: &String) -> Option<Profile> {
    let un_clone = username.clone();
    let temp = get_profiles().clone();
    let found = temp.iter().find(|x| x.name == un_clone).cloned();
    found
}
