use crate::models::platform::get_current_os;
use std::env::{home_dir, var_os};
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::create_dir_all;
use crate::models::error::Returns;
use crate::models::java::Java;

pub fn get_minecraft_directory() -> PathBuf {
    let os = get_current_os();
    match os.as_str() {
        "osx" => var_os("$HOME")
            .map(|home| PathBuf::from(home).join("Library/Application Support/minecraft"))
            .unwrap(),
        "linux" => home_dir().unwrap().join(".minecraft"),
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

pub fn get_profiles_file() -> PathBuf {
    get_falcon_launcher_directory().join("profiles.json")
}

pub fn get_temp_directory() -> PathBuf {
    get_falcon_launcher_directory().join("temp")
}

pub async fn create_necessary_dirs() {
    create_dir_all(get_versions_directory()).await.unwrap();
    create_dir_all(get_mods_folder()).await.unwrap();
    create_dir_all(get_falcon_launcher_directory())
        .await
        .unwrap();
    create_dir_all(get_assets_directory()).await.unwrap();
    create_dir_all(get_launcher_java_directory()).await.unwrap();
}

pub fn version_manifest_directory() -> PathBuf {
    get_versions_directory()
        .join("version_manifest_v2.json")
}
pub fn get_config_directory() -> PathBuf {
    get_falcon_launcher_directory().join("launcher-settings.ini")
}
fn validate_java(path: PathBuf) -> bool{

    let java_file = if get_current_os() == "windows" {
        "java.exe"
    } else {
        "java"
    };
    path.join("bin").join(java_file).exists()

}
pub fn auto_detect_javas() -> Returns<Vec<Java>> {
    let mut paths = Vec::new();
    let dirs = if get_current_os() == "windows" {
       vec![
            r"C:\Program Files\Java",
            r"C:\Program Files (x86)\Java",
        ]
    }else if get_current_os() == "linux" {
        vec![
            "/usr/lib/jvm",
            "/usr/java",
            "/usr/local/java",
        ]
    } else {
        vec!["/Library/Java/JavaVirtualMachines"]
    };
    for path in dirs.iter().map(PathBuf::from) {
        if path.read_dir().is_err() {
            continue;
        }
        for path in path.read_dir().unwrap() {
            if path.is_err() {continue;}
            let path = path.unwrap().path();
            if validate_java(path.clone()) {
                paths.push(Java::new(path));
            }
        }
    }
    Ok(paths)
}
pub fn get_java_dir() -> PathBuf {
    get_minecraft_directory().join("runtime")
}