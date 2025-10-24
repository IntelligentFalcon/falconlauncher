use crate::directory_manager::{get_libraries_directory, get_versions_directory};
use crate::structs;
use crate::structs::MinecraftVersion;
use crate::version_manager::{
    download_version_manifest, load_version_manifest_local, Manifest, VersionType,
};
use reqwest::Client;
use serde_json::Value;
use std::fs::File;
use std::path::Path;
use std::thread::spawn;
use std::time::Duration;
use tokio::fs::create_dir_all;

pub fn get_current_os() -> String {
    structs::parse_os(sys_info::os_type().expect("Unsupported Operating System"))
}
fn load_downloaded_versions() {
    let dir = get_versions_directory();
    let folders = dir.read_dir();
    match folders {
        Ok(folders) => {
            for folder in folders.filter(|x| x.is_ok()).map(|x| x.unwrap()) {
                folder
                    .path()
                    .read_dir()
                    .unwrap()
                    .map(|x| x.unwrap())
                    .filter(|x| x.path().extension().unwrap() == "json")
                    .for_each(|x| {})
            }
        }

        _ => {
            /// TODO: Handle error on no version folder found in .minecraft
            spawn(|| async move {
                create_dir_all(dir).await;
            });
            return;
        }
    }
}
pub fn get_downloaded_versions() -> Vec<MinecraftVersion> {
    if !get_versions_directory().exists() {
        return Vec::new();
    }
    get_versions_directory()
        .read_dir()
        .unwrap()
        .map(|x| x.unwrap())
        .filter(|x| {
            if x.path().is_file() {
                return false;
            }
            let children_files = x.path().read_dir().unwrap();
            ///TEMPORARY === 1 MUST BE CHANGED FOR MODPACK COMPATIBILITY
            return children_files
                .map(|f| f.unwrap())
                .filter(|f| f.path().is_file() && f.path().extension().unwrap() == "json")
                .count()
                == 1;
        })
        .map(|v| {
            MinecraftVersion::from_folder(
                get_versions_directory().join(v.file_name().to_str().unwrap().to_string()),
            )
        })
        .collect()
}
/// Loads downloaded versions and non-downloaded versions (if it is connected to the internet)
pub async fn load_versions(snapshots: bool, old_versions: bool) -> Vec<MinecraftVersion> {
    let mut versions = Vec::new();
    if !get_versions_directory().exists() {
        std::fs::create_dir(get_versions_directory()).unwrap();
    }
    let mut filtered_types = vec![VersionType::Release];
    if snapshots {
        filtered_types.push(VersionType::Snapshot);
    }
    if old_versions {
        filtered_types.push(VersionType::OldAlpha);
        filtered_types.push(VersionType::OldBeta);
    }

    versions = get_versions_directory()
        .read_dir()
        .unwrap()
        .map(|x| x.unwrap())
        .filter(|x| {
            if x.path().is_file() {
                return false;
            }
            let children_files = x.path().read_dir().unwrap();
            return children_files
                .map(|f| f.unwrap())
                .filter(|f| f.path().is_file() && f.path().extension().unwrap() == "json")
                .count()
                > 0;
        })
        .map(|v| {
            MinecraftVersion::from_folder(
                get_versions_directory().join(v.file_name().to_str().unwrap().to_string()),
            )
        })
        .collect();
    if get_versions_directory()
        .join("version_manifest_v2.json")
        .exists()
    {
        let json = load_version_manifest_local().await;
        let founded_versions = match json {
            None => Vec::new(),
            Some(v) => load_versions_through_json(v, filtered_types),
        };
        versions = extend_once(versions, founded_versions);
    }
    versions
}

fn load_versions_through_json(v: Manifest, types: Vec<VersionType>) -> Vec<MinecraftVersion> {
    let versions = v.versions;
    versions
        .iter()
        .filter(|ver| types.contains(&ver.version_type))
        .map(|x| MinecraftVersion::from_id(x.id.clone()))
        .collect()
}

/// Verifies if file exists and is not broken by the expected file size if expected_size is zero it will ignore checking file size
pub fn verify_file_existence(path_str: &String, expected_size: u64) -> bool {
    let path = Path::new(&path_str);
    if !path.exists() {
        false
    } else if expected_size != 0 {
        let file = File::open(path).expect(&("Error ".to_string() + path_str));
        let metadata = file.metadata().unwrap();
        metadata.len() == expected_size
    } else {
        true
    }
}
pub async fn is_connected_to_internet() -> bool {
    let client = Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();

    let req = client.get("https://google.com").send().await;

    match req {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub async fn load_json_url(url: &String) -> Option<Value> {
    let result = reqwest::get(url).await.unwrap();
    let text = result.text().await.unwrap_or(String::new());
    Some(serde_json::from_str(text.as_str()).expect("JSON File isn't well formatted."))
}

pub fn vec_to_string(vec: Vec<String>, separator: String) -> String {
    let mut builder = "".to_string();
    for s in vec {
        builder.push_str(&s);
        builder.push_str(&separator);
    }
    builder.remove(builder.len() - 1);
    builder
}

pub fn parse_library_name_to_path(mavenized_path: String) -> String {
    let parts = mavenized_path.split(":").collect::<Vec<&str>>();
    let group = parts[0].replace(".", "/");
    let artifact_id = parts[1];
    let version = parts[2];
    format!(
        "{}/{group}/{artifact_id}/{version}/{artifact_id}-{version}.jar",
        get_libraries_directory().to_str().unwrap()
    )
}

/// concatenate two vectors without adding repeated indexes
pub fn extend_once<T: PartialEq>(mut vec1: Vec<T>, vec2: Vec<T>) -> Vec<T> {
    for index in vec2 {
        if !vec1.contains(&index) {
            vec1.push(index);
        }
    }
    vec1
}
pub fn convert_to_full_url(base_url: String, library_name: String) -> String {
    let args = library_name.split(":").collect::<Vec<_>>();
    let group_id = args[0].replace(".", "/");
    let artifact_id = args[1];
    let version = args[2];
    let artifact_version = format!("{artifact_id}-{version}");
    format!(
        "{}{}/{}/{}/{}.jar",
        base_url, group_id, artifact_id, version, artifact_version
    )
}
pub fn convert_to_full_path(base_path: String, library_name: &String) -> String {
    let args = library_name.split(":").collect::<Vec<_>>();
    let group_id = args[0].replace(".", "/");
    let artifact_id = args[1];
    let version = args[2];
    let artifact_version = format!("{artifact_id}-{version}");
    format!(
        "{}/{}/{}/{}/{}.jar",
        base_path, group_id, artifact_id, version, artifact_version
    )
}

pub fn get_core_version(version_id: &String) -> String {
    let args = version_id.split(".").collect::<Vec<_>>();
    format!("{}.{}", args[0], args[1])
}
