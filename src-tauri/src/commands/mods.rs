use crate::models::error::{AppError, Void};
use crate::models::mods::ModInfo;
use crate::services::directory_manager::get_mods_folder;
use crate::services::mod_manager;
use crate::services::mod_manager::{load_mod, set_mod_enabled};
use log::info;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{command, AppHandle};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
use tokio::fs::copy;
use zip::ZipArchive;
use crate::models::error::AppError::InvalidPath;

#[command]
pub async fn toggle_mod(mod_info: ModInfo, toggle: bool) -> Result<(), AppError> {
    set_mod_enabled(mod_info, toggle)
}

#[command]
pub async fn get_mods() -> Result<Vec<ModInfo>, AppError> {
    let mut mods_vec: Vec<ModInfo> = Vec::new();
    let mods_directory = get_mods_folder();
    let allowed_ext = vec!["jar", "jar.disabled", "disabled"];
    let mod_list = mods_directory
        .read_dir()
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|x| {
            x.as_path().is_file()
                && allowed_ext.contains(&x.as_path().extension().unwrap().to_str().unwrap())
        })
        .collect::<Vec<PathBuf>>();
    for jar_file in mod_list {
        let zip = ZipArchive::new(File::open(jar_file.clone()).unwrap()).unwrap();
        let Ok(loaded) = load_mod(Mutex::new(zip), jar_file.to_str().unwrap().to_string()) else {
            continue;
        };
        mods_vec.push(loaded);
    }
    Ok(mods_vec)
}
#[command]
pub async fn import_mod_from_local(app: AppHandle) -> Result<(), AppError> {
    let paths = app
        .dialog()
        .file()
        .add_filter("Minecraft mods".to_string(), &[&"jar", &"disabled"])
        .blocking_pick_files()
        .unwrap_or_default();
    for path in paths.iter().filter_map(|x| x.as_path()) {
        let Some(file_name) = path.file_name() else {
            continue;
        };
        let new_path = get_mods_folder().join(file_name);
        copy(path, new_path)
            .await
            .map_err(|x| AppError::FileCopyFailed(x.to_string()))?;
    }
    Ok(())
}

#[command]
pub async fn delete_mod(mod_info: ModInfo) -> Result<(), AppError> {
    let path = PathBuf::from(&mod_info.path);
    fs::remove_file(&path).map_err(|x| AppError::FileDeleteFailed(x.to_string()))
}

#[command]
pub async fn open_mods_folder(app: AppHandle,version: String) -> Void{
    let mods_dir = get_mods_folder();
    let mods_dir_str = mods_dir.to_str().ok_or(InvalidPath(format!("{version}'s mod directory")))?;
    app.opener().open_path(mods_dir_str, None::<&str>)
        .map_err(|e| AppError::OpenPathFailed(format!("failed to open the {version}'s mod directory: {e}")))
}