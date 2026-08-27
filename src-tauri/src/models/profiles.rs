use crate::models::error::{AppError, Void};
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
        let res = File::create(&get_profiles_file()).map_err(|e| AppError::FileCreateFailed(e.to_string()))?;
    }
    let uuid = uuid_from_username(username.as_str());
    // TODO: implement online profile as well (Profile::Microsoft)
    // if online { ...
    profiles.push(Profile {
        username,
        online,
        uuid,
    });

    let json_string = serde_json::to_string_pretty(&profiles).map_err(|e| AppError::JsonParseFailed(e.to_string()))?;
    fs::write(get_profiles_file(), json_string)
        .map_err(|e| AppError::FileWriteFailed(format!("Failed to write on file {}: {}", get_profiles_file().to_string_lossy(),
                                                       e.to_string())))?;
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

pub fn get_profile(uuid: &Uuid) -> Option<Profile> {
    let uid_clone = uuid.clone();
    let temp = get_profiles().clone();
    let found = temp.iter().find(|x| x.uuid == uid_clone).cloned();
    found
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub username: String,
    pub online: bool,
    pub uuid: Uuid,
}

impl Default for Profile {
    fn default() -> Self {
        Self{
            username: "Player".to_string(),
            online: false,
            uuid: uuid_from_username("Player"),
        }
    }
}