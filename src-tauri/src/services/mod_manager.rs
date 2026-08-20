use crate::services::directory_manager::get_mods_folder;
use crate::models::error::AppError;
use crate::models::mods::{FabricModInfo, McModInfo};
use crate::models::mods::ModInfo;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Mutex;
use log::info;
use toml::Value;
use zip::ZipArchive;

pub fn set_mod_enabled(m: ModInfo, toggle: bool) -> Result<(), AppError> {
    let mut path = PathBuf::from(&m.path);
    let new_path = if toggle {
        let mut new = path.clone();
        path.set_extension("disabled");
        new.set_extension("jar");
        new
    } else {
        let mut new = path.clone();
        path.set_extension("jar");
        new.set_extension("disabled");
        new
    };
    fs::rename(&path, &new_path).map_err(|x| AppError::FileRenameFailed(x.to_string()))
}

pub fn load_mod(zip: Mutex<ZipArchive<File>>, path: String) -> Result<ModInfo, AppError> {
    let enabled = path.to_lowercase().ends_with("jar");

    let mut zip_guard = zip
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire lock on zip archive".to_string()))?;

    info!("Loading mod: {}", path);

    // Forge legacy versions
    if let Ok(mut mod_info) = zip_guard.by_name("mcmod.info") {
        let mut content = String::new();
        mod_info
            .read_to_string(&mut content)
            .map_err(|e| AppError::FileReadFailed(format!("Failed to read mcmod.info: {}", e)))?;

        let mcmods: Vec<McModInfo> = serde_json::from_str(&content)
            .map_err(|e| AppError::JsonParseFailed(format!("Failed to parse mcmod.info: {}", e)))?;

        let mcmod_info = mcmods
            .first()
            .ok_or_else(|| AppError::JsonParseFailed("mcmod.info is empty".to_string()))?;

        return Ok(ModInfo {
            path,
            mod_id: mcmod_info.mod_id.clone(),
            name: mcmod_info.name.clone(),
            version: mcmod_info.version.clone(),
            description: mcmod_info.description.clone(),
            enabled,
        });
    }

    // Forge new versions
    if let Ok(mut file) = zip_guard.by_name("META-INF/mods.toml") {
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| AppError::FileReadFailed(format!("Failed to read mods.toml: {}", e)))?;

        let toml: Value = toml::from_str(content.as_str())
            .map_err(|e| AppError::ModLoadingFailed(format!("Failed to parse mods.toml: {}", e)))?;

        return load_from_toml(&toml, path);
    }

    // Neoforge
    if let Ok(mut file) = zip_guard.by_name("META-INF/neoforge.mods.toml") {
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| AppError::FileReadFailed(format!("Failed to read neoforge.mods.toml: {}", e)))?;

        let toml: Value = toml::from_str(content.as_str())
            .map_err(|e| AppError::ModLoadingFailed(format!("Failed to parse neoforge.mods.toml: {}", e)))?;

        return load_from_toml(&toml, path);
    }

    // Fabric
    if let Ok(mut file) = zip_guard.by_name("fabric.mod.json") {
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| AppError::FileReadFailed(format!("Failed to read fabric.mod.json: {}", e)))?;

        let info: FabricModInfo = serde_json::from_str(content.as_str())
            .map_err(|e| AppError::JsonParseFailed(format!("Failed to parse fabric.mod.json: {}", e)))?;

        return Ok(ModInfo {
            path,
            mod_id: info.mod_id.clone(),
            name: info.name.clone(),
            version: info.version.clone(),
            description: info.description.clone(),
            enabled,
        });
    }

    Err(AppError::ModLoadingFailed("No valid mod metadata found in archive".to_string()))
}

fn load_from_toml(toml: &Value, path: String) -> Result<ModInfo, AppError> {
    let mod_array = toml
        .get("mods")
        .and_then(|m| m.as_array())
        .ok_or_else(|| AppError::ModLoadingFailed("Missing 'mods' array in TOML".to_string()))?;

    let mut mod_id = String::new();
    let mut display_name = String::new();
    let mut version = String::new();
    let mut desc = String::new();

    mod_array.iter().for_each(|index| {
        if let Some(id) = index.get("modId").and_then(|x| x.as_str()) {
            mod_id = id.to_string();
        }
        if let Some(description) = index.get("description").and_then(|x| x.as_str()) {
            desc = description.to_string();
        }
        if let Some(ver) = index.get("version").and_then(|x| x.as_str()) {
            version = ver.to_string();
        }
        if let Some(disp_name) = index.get("displayName").and_then(|x| x.as_str()) {
            display_name = disp_name.to_string();
        }
    });

    Ok(ModInfo::new(path, mod_id, display_name, version, desc))
}