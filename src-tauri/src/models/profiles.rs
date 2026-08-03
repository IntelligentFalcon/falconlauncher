use crate::models::error::AppError;
use crate::services::directory_manager::get_profiles_file;
use crate::services::utils::uuid_from_username;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs;
use std::fs::{read_to_string, File};
use serde::ser::SerializeStruct;
use uuid::Uuid;

pub fn create_new_profile(username: String, online: bool) -> Result<(), AppError> {
    let mut profiles = get_profiles();
    let result = Ok(());
    if !get_profiles_file().exists() {
        let res = File::create(&get_profiles_file());
        if res.is_err() {
            return Err(AppError::FileCreateFailed(res.err().unwrap().to_string()));
        }
    }
    let uuid = uuid_from_username(username.as_str());
    // TODO: implement online profile as well (Profile::Microsoft)
    // if online { ...
    profiles.push(Profile {
        username,
        online,
        uuid,
    });

    let json_string = serde_json::to_string_pretty(&profiles);
    if json_string.is_err() {
        return Err(AppError::JsonParseFailed(json_string.err().unwrap().to_string()));
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
    let found = temp.iter().find(|x| x.username == un_clone).cloned();
    found
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub username: String,
    pub online: bool,
    pub uuid: Uuid,

}