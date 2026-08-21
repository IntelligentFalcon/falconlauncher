use crate::models::error::{AppError, Void};
use crate::models::java::Java;
use crate::models::platform::get_current_os;
use std::env::{home_dir, var_os};
use std::path::PathBuf;
use tokio::fs::create_dir_all;
use crate::models::mirror::{mojang_mirror, ninecraft_mirror};

pub fn get_minecraft_directory() -> PathBuf {
    let os = get_current_os();
    match os.as_str() {
        "osx" => var_os("HOME")
            .map(|home| PathBuf::from(home).join("Library/Application Support/minecraft"))
            .unwrap_or_else(|| PathBuf::from(".minecraft")),

        "linux" => home_dir()
            .map(|home| home.join(".minecraft"))
            .unwrap_or_else(|| PathBuf::from(".minecraft")),

        _ => var_os("APPDATA")
            .map(|home| PathBuf::from(home).join(".minecraft"))
            .unwrap_or_else(|| PathBuf::from(".minecraft")),
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

pub fn get_natives_directory(version: &String) -> PathBuf {
    get_version_directory(version).join("natives")
}

pub fn get_assets_directory() -> PathBuf {
    get_minecraft_directory().join("assets")
}

pub fn get_falcon_launcher_directory() -> PathBuf {
    get_minecraft_directory().join("falconlauncher")
}

pub fn get_mods_directory() -> PathBuf {
    get_minecraft_directory().join("mods")
}

pub fn get_profiles_file() -> PathBuf {
    get_falcon_launcher_directory().join("profiles.json")
}

pub fn get_temp_directory() -> PathBuf {
    get_falcon_launcher_directory().join("temp")
}

pub async fn create_necessary_dirs() -> Void {
    create_dir_all(get_versions_directory()).await.map_err(|x| AppError::DirCreateFailed(x.to_string()))?;
    create_dir_all(get_mods_directory()).await.map_err(|x| AppError::DirCreateFailed(x.to_string()))?;
    create_dir_all(get_falcon_launcher_directory())
        .await
        .map_err(|x| AppError::DirCreateFailed(x.to_string()))?;
    create_dir_all(get_assets_directory()).await.map_err(|x| AppError::DirCreateFailed(x.to_string()))?;
    create_dir_all(get_java_dir()).await.map_err(|x| AppError::DirCreateFailed(x.to_string()))?;
    create_dir_all(get_mirrors_dir()).await.map_err(|x| AppError::DirCreateFailed(x.to_string()))?;
    mojang_mirror().write().map_err(|x| AppError::FileWriteFailed(x.to_string()))?;

    Ok(())
}

pub fn version_manifest_directory() -> PathBuf {
    get_versions_directory().join("version_manifest_v2.json")
}

pub fn get_config_directory() -> PathBuf {
    get_falcon_launcher_directory().join("launcher-settings.ini")
}

fn validate_java(path: PathBuf) -> bool {
    let java_file = if get_current_os() == "windows" {
        "java.exe"
    } else {
        "java"
    };
    path.join("bin").join(java_file).exists()
}

pub fn auto_detect_javas() -> Result<Vec<Java>, AppError> {
    let mut paths = Vec::new();
    let dirs = if get_current_os() == "windows" {
        vec![
            r"C:\Program Files\Java",
            r"C:\Program Files (x86)\Java",
        ]
    } else if get_current_os() == "linux" {
        vec![
            "/usr/lib/jvm",
            "/usr/java",
            "/usr/local/java",
        ]
    } else {
        vec!["/Library/Java/JavaVirtualMachines"]
    };

    for path in dirs.iter().map(PathBuf::from) {
        let Ok(read_dir) = path.read_dir() else {
            continue;
        };
        for entry in read_dir.filter_map(Result::ok) {
            let entry_path = entry.path();
            if validate_java(entry_path.clone()) {
                paths.push(Java::new(entry_path));
            }
        }
    }
    Ok(paths)
}

pub fn get_java_dir() -> PathBuf {
    get_minecraft_directory().join("runtime")
}

pub fn get_mirrors_dir() -> PathBuf {
    get_falcon_launcher_directory().join("mirrors")
}

pub fn get_version_manifest(id: &String) -> PathBuf {
    get_version_directory(id).join(format!("{}.json", id))
}