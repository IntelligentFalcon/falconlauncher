use crate::models::error::{AppError, Void};
use crate::models::profiles;
use crate::models::profiles::Profile;
use crate::services::directory_manager::get_profiles_file;
use crate::{AppState, GLOBAL_CACHE};
use std::fs;
use tauri::{command, AppHandle, Manager, State};
use uuid::Uuid;
use crate::services::utils::uuid_from_username;

#[command]
pub async fn get_profiles() -> Result<Vec<Profile>, AppError> {
    Ok(profiles::get_profiles())
}

#[command]
pub async fn create_offline_profile(
    state: State<'_, AppState>,
    username: String,
) -> Result<(), AppError> {
    let mut cfg = state.config.write().await;
    let result = profiles::create_new_profile(username.clone(), false);
    cfg.launch_options.selected_profile = uuid_from_username(username.as_str());
    result
}

#[command]
pub async fn remove_profile(state: State<'_, AppState>, profile: Profile) -> Void {
    if !get_profiles_file().exists() {
        return Ok(());
    }
    let profiles = profiles::get_profiles();
    let filtered_profiles = profiles
        .iter()
        .filter(|x| x.uuid != profile.uuid)
        .collect::<Vec<&Profile>>();
    let json = serde_json::to_string(&filtered_profiles);
    let cfg = state.config.write().await;

    fs::write(
        get_profiles_file(),
        json.map_err(|x| AppError::JsonParseFailed(x.to_string()))?,
    )
    .map_err(|x| AppError::FileWriteFailed(x.to_string()))?;
    if cfg.launch_options.selected_profile == profile.uuid {

    }
    Ok(())
}
