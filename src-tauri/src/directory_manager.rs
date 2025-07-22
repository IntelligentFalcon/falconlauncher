use crate::utils::get_current_os;
use std::env::var_os;
use std::path::PathBuf;

pub fn get_minecraft_directory() -> PathBuf {
    let os = get_current_os();
    match os.as_str() {
        "osx" => var_os("$HOME")
            .map(|home| PathBuf::from(home).join("Library/Application Support/minecraft"))
            .unwrap(),
        "linux" => var_os("APPDATA")
            .map(|appdata| PathBuf::from(appdata).join(".minecraft"))
            .unwrap(),
        _ => var_os("APPDATA")
            .map(|home| PathBuf::from(home).join(".minecraft"))
            .unwrap(),
    }
}
pub fn get_libraries_directory() -> PathBuf {
    get_minecraft_directory().join("libraries")
}

pub fn get_versions_directory() -> PathBuf {
    get_minecraft_directory().join("versions")
}
pub fn get_version_directory(version: &String) -> PathBuf {
    get_versions_directory().join(version)
}
pub fn get_natives_folder(version: &String) -> PathBuf {
    get_version_directory(version).join("natives")
}
pub fn get_assets_directory() -> PathBuf {
    get_minecraft_directory().join("assets")
}
pub fn get_falcon_launcher_directory() -> PathBuf {
    get_minecraft_directory().join("falconlauncher")
}

pub fn get_launcher_java_directory() -> PathBuf {
    get_falcon_launcher_directory().join("java")
}

pub fn get_mods_folder() -> PathBuf {
    get_minecraft_directory().join("mods")
}
