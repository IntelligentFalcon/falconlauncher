use std::fs;
use std::path::PathBuf;
use log::info;
use crate::models::error::AppError;
use crate::models::mods::ModInfo;
use crate::services::directory_manager::get_mods_folder;
use crate::services::mod_manager;
use crate::services::mod_manager::{load_mods, set_mod_enabled};
use tauri::{command, AppHandle};
use tauri_plugin_dialog::DialogExt;
use tokio::fs::copy;

#[command]
pub async fn toggle_mod(mod_info: ModInfo, toggle: bool) -> Result<(), AppError> {
    set_mod_enabled(mod_info, toggle)
    
}

#[command]
pub async fn get_mods() -> Result<Vec<ModInfo>, AppError> {
    Ok(load_mods())
}

#[command]
pub async fn import_mod_from_local(app: AppHandle) -> Result<(), AppError> {
    let paths = app
        .dialog()
        .file()
        .add_filter("Minecraft mods".to_string(), &[&"jar", &"disabled"])
        .blocking_pick_files().unwrap_or_default();
    for path in paths.iter().filter_map(|x| x.as_path()) {
        let Some(file_name) = path.file_name() else {
            continue;
        };
        let new_path = get_mods_folder().join(file_name);
        copy(path, new_path).await.map_err(|x| AppError::FileCopyFailed(x.to_string()))?;
    }
    Ok(())
}

#[command]
pub async  fn delete_mod(mod_info: ModInfo) -> Result<(), AppError> {
    let path = PathBuf::from(&mod_info.path);
    fs::remove_file(&path).map_err(|x| AppError::FileDeleteFailed(x.to_string()))
}