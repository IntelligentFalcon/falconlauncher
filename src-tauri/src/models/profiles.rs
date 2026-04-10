use crate::services::directory_manager::get_profiles_file;
use crate::models::error::{io_err_create_file, json_read_err, InvokeError};
use std::fs;
use std::fs::{read_to_string, File};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

pub fn create_new_profile(username: String, online: bool) -> Result<(), InvokeError<()>> {
    let mut profiles: Vec<Profile> = get_profiles();
    let result = Ok(());
    if !get_profiles_file().exists() {
        let res = File::create(&get_profiles_file());
        if res.is_err() {
            return Err(io_err_create_file(
                get_profiles_file().to_str().unwrap().to_string(),
                res.err().unwrap(),
            ));
        }
    }
    let uid = Uuid::new_v4();

    profiles.push(Profile {
        name: username,
        online,
        uuid: uid,
    });

    let json_string = serde_json::to_string_pretty(&profiles);
    if json_string.is_err() {
        return Err(json_read_err(json_string.err().unwrap()));
    }
    fs::write(get_profiles_file(), json_string.unwrap()).expect("Failed to write the file!");
    result
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub online: bool,
    pub uuid: uuid::Uuid,
}