use crate::models::error::{io_err_buffer_read, io_err_read_file, Returns};
use crate::models::platform::get_current_os;
use crate::services::directory_manager::get_libraries_directory;
use reqwest::Client;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use crate::models::java::Java;

fn calculate_file_sha1<P: AsRef<Path>>(path: P) -> Returns<String> {
    let file = File::open(path).map_err(|e| io_err_read_file(e))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192]; // Read in 8KB chunks

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|x| io_err_buffer_read(x))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
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

pub async fn load_json_url(url: &String) -> Option<Value> {
    let result = reqwest::get(url). await.unwrap();
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

pub fn can_apply_rule(rule_obj: &Map<String, Value>) -> bool {
    let rules = match rule_obj.get("rules").and_then(|r| r.as_array()) {
        Some(r) => r,
        None => return true,
    };

    let mut allowed = false;

    for rule in rules {
        if let Some(rule_map) = rule.as_object() {
            let action = rule_map.get("action").and_then(|a| a.as_str()).unwrap_or("allow");
            let applies = check_os_rule(rule_map);
            if applies {
                match action {
                    "allow" => allowed = true,
                    "disallow" => return false,
                    _ => {}
                }
            }
        }
    }

    allowed
}

pub fn check_os_rule(rule_map: &Map<String, Value>) -> bool {
    let os_condition = match rule_map.get("os") {
        Some(os) => os.as_object(),
        None => return true,
    };

    let Some(os) = os_condition else { return true };

    if let Some(name) = os.get("name").and_then(|n| n.as_str()) {
        let current_os = get_current_os();
        return current_os.to_string() == name.to_string();
    }
    true

}

pub fn update_download_bar(progress: i64, app_handle: &AppHandle) {
    app_handle.emit("progressBar", progress).unwrap();
}
pub fn update_download_status(text: &str, app_handle: &AppHandle) {
    app_handle.emit("progress", text).unwrap();
}
pub fn update_download(progress: i64, text: &str, app_handle: &AppHandle) {
    app_handle.emit("progress", text).unwrap();
    app_handle.emit("progressBar", progress).unwrap();
}

/// Fixes permission issues related to java when using linux.
pub fn linux_java_permission_fix(java: &Java){
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(java.get_bin_file()) {
            let mut permissions = metadata.permissions();
            let current_mode = permissions.mode();

            if current_mode & 0o111 == 0 {
                println!("Adding execute permission to Java binary: {:?}", &java.get_bin_file());
                permissions.set_mode(current_mode | 0o111);
                std::fs::set_permissions(&java.get_bin_file(), permissions)
                    .expect("Failed to set execute permissions on Java binary");
            }
        } else {
            panic!("Java binary not found at: {:?}", &java.get_bin_file());
        }
    }
}