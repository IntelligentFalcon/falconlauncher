use crate::models::error::{todo_err, Returns, Void};
use crate::models::mirror::{list_mirrors, Mirror};
use crate::services::directory_manager::get_mirrors_dir;
use crate::AppState;
use std::fs;
use tauri::{command, AppHandle, State};

#[command]
pub async fn get_available_mirrors() -> Returns<Vec<Mirror>> {
    Err(todo_err("testttttttes tgasrgsdag"))
    // list_mirrors()
}

#[command]
pub async fn set_mirror(app_handle: AppHandle, state: State<'_, AppState>, mirror: Mirror) -> Void {
    state.config.write().await.download_settings.mirror = mirror;
    Ok(())
}

#[command]
pub async fn get_mirror(app_handle: AppHandle, state: State<'_, AppState>) -> Returns<Mirror> {
    Ok(state.config.read().await.download_settings.mirror.clone())
}

#[command]
pub async fn import_mirror(json: String) -> Void {
    if let Ok(value) = serde_json::from_str::<Mirror>(json.as_str()) {
        fs::write(get_mirrors_dir().join(format!("{}.json", value.name.to_lowercase())), json).map_err(|x| todo_err("Write failed"))

    } else {
        Err(todo_err("Invalid json format"))
    }
}
