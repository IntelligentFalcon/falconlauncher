use tauri::{command, State};
use crate::AppState;
use crate::models::error::{Returns, Void};
use crate::models::profiles;
use crate::models::profiles::Profile;

#[command]
pub async fn get_profiles() -> Returns<Vec<Profile>> {
    Ok(profiles::get_profiles())
}

#[command]
pub async fn create_offline_profile(state: State<'_, AppState>, username: String) -> Void {
    let mut cfg = state.config.write().await;
    let result = profiles::create_new_profile(username.clone(), false);
    cfg.launch_options.username = username;
    result
}