use crate::directory_manager::get_mods_folder;
use crate::structs::mod_identifiers::{FabricModInfo, McModInfo};
use crate::structs::ModInfo;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::iter::Zip;
use std::path::PathBuf;
use std::sync::Mutex;
use toml::{from_str, Value};
use zip::result::ZipResult;
use zip::ZipArchive;

pub fn set_mod_enabled(m: ModInfo, toggle: bool) {
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
    fs::rename(&path, &new_path).unwrap();
}
pub fn load_mod(mut zip: Mutex<ZipArchive<File>>, path: String) -> ModInfo {
    let enabled = path.to_lowercase().ends_with("jar");
    let mut zip_guard = zip.lock().unwrap();

    println!("Loading mod: {}", path);

    // Forge legacy versions
    if let Ok(mut mod_info) = zip_guard.by_name("mcmod.info") {
        let mut content = String::new();
        mod_info
            .read_to_string(&mut content)
            .expect("Failed to read file");
        let mcmods: Vec<McModInfo> = serde_json::from_str(&content).unwrap();
        let mcmod_info = &mcmods[0];

        return ModInfo {
            path,
            mod_id: mcmod_info.mod_id.clone(),
            name: mcmod_info.name.clone(),
            version: mcmod_info.version.clone(),
            description: mcmod_info.description.clone(),
            enabled,
        };
    }

    // Forge new versions
    if let Ok(mut file) = zip_guard.by_name("META-INF/mods.toml") {
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let toml: Value = toml::from_str(content.as_str()).unwrap();
        return load_from_toml(&toml, path);
    }

    // Neoforge
    if let Ok(mut file) = zip_guard.by_name("META-INF/neoforge.mods.toml") {
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let toml: Value = toml::from_str(content.as_str()).unwrap();
        return load_from_toml(&toml, path);
    }

    // Fabric
    if let Ok(mut file) = zip_guard.by_name("fabric.mod.json") {
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let info: FabricModInfo = serde_json::from_str(content.as_str()).unwrap();
        return ModInfo {
            path,
            mod_id: info.mod_id.clone(),
            name: info.name.clone(),
            version: info.version.clone(),
            description: info.description.clone(),
            enabled,
        };
    }
    // Fallback
    println!("testt");
    ModInfo {
        path: "".to_string(),
        mod_id: "".to_string(),
        name: "".to_string(),
        version: "".to_string(),
        description: "".to_string(),
        enabled,
    }
}
pub fn load_mods() -> Vec<ModInfo> {
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
        let mut zip = ZipArchive::new(File::open(jar_file.clone()).unwrap()).unwrap();
        let loaded = load_mod(Mutex::new(zip), jar_file.to_str().unwrap().to_string());
        mods_vec.push(loaded);
    }
    mods_vec
}
fn load_from_toml(toml: &Value, path: String) -> ModInfo {
    let mod_array = toml["mods"].as_array().unwrap();
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
    ModInfo::new(path, mod_id, display_name, version, desc)
}
