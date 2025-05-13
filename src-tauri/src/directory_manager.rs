use crate::utils::get_current_os;
use std::env::var_os;
use std::path::PathBuf;

pub fn get_minecraft_directory() -> Option<PathBuf> {
    let os = get_current_os();
    match os.as_str() {
        "osx" => var_os("$HOME")
            .map(|home| PathBuf::from(home).join("Library/Application Support/minecraft")),
        "linux" => var_os("APPDATA").map(|appdata| PathBuf::from(appdata).join(".minecraft")),
        _ => var_os("APPDATA").map(|home| PathBuf::from(home).join(".minecraft")),
    }
}
pub fn get_libraries_directory() -> Option<PathBuf> {
    let minecraft_dir = get_minecraft_directory();
    match minecraft_dir {
        None => None,
        Some(buf) => Some(buf.join("libraries")),
    }
}

pub fn get_versions_directory() -> Option<PathBuf> {
    let minecraft_dir = get_minecraft_directory();
    match minecraft_dir {
        None => None,
        Some(buf) => Some(buf.join("versions")),
    }
}
pub fn get_version_directory(version: &String) -> Option<PathBuf> {
    let versions_dir = get_versions_directory();
    match versions_dir {
        None => None,
        Some(buf) => Some(buf.join(version)),
    }
}
pub fn get_natives_folder(version: &String) -> Option<PathBuf> {
    match get_version_directory(version) {
        None => None,
        Some(v) => Some(v.join("natives")),
    }
}
pub fn get_assets_directory() -> Option<PathBuf> {
    let minecraft_dir = get_minecraft_directory();
    match minecraft_dir {
        None => None,
        Some(buf) => Some(buf.join("assets")),
    }
}
pub fn get_falcon_launcher_directory() -> Option<PathBuf> {
    let minecraft_directory = get_minecraft_directory();
    match minecraft_directory {
        None => None,
        Some(val) => Some(val.join("falconlauncher"))
    }
}
