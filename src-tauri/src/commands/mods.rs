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
        .blocking_pick_files()
        .unwrap();
    for path in paths {
        let p = path.as_path().unwrap();
        let file_name = p.file_name().unwrap().to_str().unwrap();
        let new_path = get_mods_folder().join(file_name);
        copy(p, new_path).await.unwrap();
    }
    Ok(())
}

#[command]
pub async fn delete_mod(mod_info: ModInfo) -> Result<(), AppError> {
    mod_manager::delete_mod(&mod_info);
    Ok(())
}