use crate::models::error::{io_err_create_file, json_read_err, Void};
use crate::services::directory_manager::get_profiles_file;
use crate::services::utils::uuid_from_username;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::{read_to_string, File};
use uuid::Uuid;

pub fn create_new_profile(username: String, online: bool) -> Void {
    let mut profiles = get_profiles();
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
    // TODO: implement online profile as well (Profile::Microsoft)
    // if online { ...
    profiles.push(ProfileNew::Offline {
        username,
    });

    let json_string = serde_json::to_string_pretty(&profiles);
    if json_string.is_err() {
        return Err(json_read_err(json_string.err().unwrap()));
    }
    fs::write(get_profiles_file(), json_string.unwrap()).expect("Failed to write the file!");
    result
}

pub fn get_profiles() -> Vec<ProfileNew> {
    serde_json::from_str(
        read_to_string(get_profiles_file())
            .unwrap_or("".to_string())
            .as_str(),
    )
    .unwrap_or(Vec::new())
}

pub fn get_profile(username: &String) -> Option<ProfileNew> {
    let un_clone = username.clone();
    let temp = get_profiles().clone();
    let found = temp.iter().find(|x| x.username() == un_clone).cloned();
    found
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ProfileNew {
    Microsoft {
        username: String,
        uuid: Uuid,
        access_token: String,
        refresh_token: String,
    },
    Offline {
        username: String,
    },
}

impl ProfileNew {
    pub fn uuid(&self) -> Uuid {
        match self {
            ProfileNew::Microsoft { uuid, .. } => *uuid,
            ProfileNew::Offline { username } => uuid_from_username(username),
        }
    }

    pub fn username(&self) -> &str {
        match self {
            ProfileNew::Microsoft { username, .. } => username,
            ProfileNew::Offline { username } => username,
        }
    }
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub online: bool,
    pub uuid: Uuid,
}