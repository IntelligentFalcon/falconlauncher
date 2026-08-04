use crate::models::error::AppError;
use crate::models::mirror::{list_mirrors, Mirror};
use crate::services::directory_manager::get_mirrors_dir;
use crate::AppState;
use std::fs;
use tauri::{command, AppHandle, State};

#[command]
pub async fn get_available_mirrors() -> Result<Vec<Mirror>, AppError> {
    list_mirrors()
}

#[command]
pub async fn set_mirror(app_handle: AppHandle, state: State<'_, AppState>, mirror: Mirror) -> Result<(), AppError> {
    state.config.write().await.download_settings.mirror = mirror;
    println!("test");
    Err(AppError::UnknownError("testttttttes tgasrgsdag".to_string()))
}

#[command]
pub async fn get_mirror(app_handle: AppHandle, state: State<'_, AppState>) -> Result<Mirror, AppError> {
    Ok(state.config.read().await.download_settings.mirror.clone())
}

#[command]
pub async fn import_mirror(json: String) -> Result<(), AppError> {
    if let Ok(value) = serde_json::from_str::<Mirror>(json.as_str()) {
        fs::write(get_mirrors_dir().join(format!("{}.json", value.name.to_lowercase())), json).map_err(|x| AppError::FileWriteFailed("Write failed".to_string()))

    } else {
        Err(AppError::JsonParseFailed("Invalid json format".to_string()))
    }
}
