use crate::directory_manager::get_mods_folder;
use crate::structs::ModInfo;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use toml::Value;

pub fn load_mods() -> Vec<ModInfo> {
    let mut mods_vec: Vec<ModInfo> = Vec::new();
    let mods_directory = get_mods_folder();
    let mod_list = mods_directory
        .read_dir()
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|x| x.as_path().is_file() && x.as_path().extension().unwrap() == "jar")
        .collect::<Vec<PathBuf>>();
    for jar_file in mod_list {
        let mut zip = zip::ZipArchive::new(File::open(jar_file.clone()).unwrap()).unwrap();
        if let Ok(mut file) = zip.by_name("META-INF/mods.toml") {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            let toml: Value = toml::from_str(content.as_str()).unwrap();
            let info = load_from_toml(&toml, jar_file.as_path().to_str().unwrap().to_string());
            mods_vec.push(info);
        };
        if let Ok(mut file) = zip.by_name("fabric.mod.json") {};
    }
    mods_vec
}
fn load_from_toml(toml: &Value, path: String) -> ModInfo {
    let mod_array = toml["mods"].as_array().unwrap();
    let mut mod_id = String::new();
    let mut display_name = String::new();
    let mut version = String::new();
    mod_array.iter().for_each(|index| {
        if let Some(id) = index.get("modId").and_then(|x| x.as_str()) {
            mod_id = id.to_string();
        }
        if let Some(ver) = index.get("version").and_then(|x| x.as_str()) {
            version = ver.to_string();
        }
        if let Some(disp_name) = index.get("displayName").and_then(|x| x.as_str()) {
            display_name = disp_name.to_string();
        }
    });
    ModInfo::new(path, mod_id, display_name, version)
}
