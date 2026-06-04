use crate::models::error::{Returns, Void};
use crate::models::mirror::Mirror;
use crate::models::mirror::{mojang_mirror, ninecraft_mirror};
use crate::services::directory_manager::get_mirrors_dir;
use std::fs;
use tauri::{command, AppHandle};

#[command]
pub async fn get_available_mirrors() -> Returns<Vec<Mirror>> {
    let mirrors_dir = get_mirrors_dir();
    let mut vec = Vec::new();
    for entry in mirrors_dir.read_dir().unwrap() {
        if let Ok(entry) = entry {
            if let Ok(content) = fs::read_to_string(get_mirrors_dir().join(entry.file_name())){
                if let Ok(value) = serde_json::from_str::<Mirror>(content.as_str()) {
                    vec.push(value);
                }
            }
        }
    }
    if vec.iter().find(|x| x.name == mojang_mirror().name).is_none(){
       vec.push(mojang_mirror())
    }
    Ok(vec)
}

#[command]
pub async fn set_mirror(app_handle: AppHandle, mirror: Mirror) -> Void {
    Ok(())
}

#[command]
pub async fn get_mirror(app_handle: AppHandle) -> Returns<Mirror> {
    Ok(ninecraft_mirror())
}

#[command]
pub async fn import_mirror(json: String) -> Result<(), String> {
    if let Ok(value) = serde_json::from_str::<Mirror>(json.as_str()) {
        fs::write(get_mirrors_dir().join(format!("{}.json",value.name.to_lowercase())), json);
        Ok(())
    } else {
        Err("Invalid json format".to_string())
    }
}
