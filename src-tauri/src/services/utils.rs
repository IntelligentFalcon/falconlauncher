use std::env;
use crate::models::downloader::{LibraryRules, Rule, RuleOS};
use crate::models::error::AppError;
use crate::models::java::Java;
use crate::models::platform::get_current_os;
use crate::services::directory_manager::get_libraries_directory;
use serde_json::{Map, Value};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Duration;
use log::info;
use reqwest::{Client, Proxy};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use sha1::{Digest, Sha1};
use sha1::digest::FixedOutput;
use tauri::{AppHandle, Emitter};
use tauri::ipc::RuntimeCapability;
use uuid::{Builder, Uuid};
use crate::models::config::Config;

pub fn verify_file_existence_with_size(path_str: &String, expected_size: u64) -> Result<bool, AppError> {
    let path = Path::new(&path_str);
    if !path.exists() {
        Ok(false)
    } else if expected_size != 0 {
        let file = File::open(path).map_err(|e| AppError::FileReadFailed(format!("Error opening {}: {}", path_str, e)))?;
        let metadata = file.metadata().map_err(|e| AppError::FileReadFailed(format!("Error reading metadata for {}: {}", path_str, e)))?;
        Ok(metadata.len() == expected_size)
    } else {
        Ok(true)
    }
}

pub fn verify_file_existence_with_sha<P: AsRef<Path>>(
    file_path: P,
    expected_sha: &str,
) -> Result<bool, AppError> {
    let path = file_path.as_ref();

    if !path.exists() {
        return Ok(false);
    }

    let mut file = File::open(path).map_err(|e| {
        AppError::InvalidPath(e.to_string())
    })?;

    let mut hasher = Sha1::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).map_err(|e| {
            AppError::FileReadFailed(format!("Failed to read file chunk: {}", e))
        })?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    let computed_hash = result.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    Ok(computed_hash.eq_ignore_ascii_case(expected_sha))
}
pub fn vec_to_string(vec: Vec<String>, separator: String) -> String {
    if vec.is_empty() {
        return String::new();
    }
    let mut builder = "".to_string();
    for s in vec {
        builder.push_str(&s);
        builder.push_str(&separator);
    }
    builder.truncate(builder.len() - separator.len());
    builder
}

pub fn parse_library_name_to_path(mavenized_path: String) -> Result<String, AppError> {
    let parts = mavenized_path.split(':').collect::<Vec<&str>>();
    if parts.len() < 3 {
        return Err(AppError::InvalidPath(format!("Invalid mavenized path: {}", mavenized_path)));
    }
    let group = parts[0].replace('.', "/");
    let artifact_id = parts[1];
    let version = parts[2];
    let libs_dir = get_libraries_directory();
    let libraries_path = libs_dir.to_str().ok_or_else(|| AppError::InvalidPath("Libraries directory".to_string()))?;
    Ok(format!(
        "{}/{group}/{artifact_id}/{version}/{artifact_id}-{version}.jar",
        libraries_path
    ))
}

pub fn extend_once<T: PartialEq>(mut vec1: Vec<T>, vec2: Vec<T>) -> Vec<T> {
    for index in vec2 {
        if !vec1.contains(&index) {
            vec1.push(index);
        }
    }
    vec1
}

pub fn convert_to_full_url(base_url: String, library_name: String) -> Result<String, AppError> {
    let args = library_name.split(':').collect::<Vec<_>>();
    if args.len() < 3 {
        return Err(AppError::InvalidPath(format!("Invalid library name: {}", library_name)));
    }
    let group_id = args[0].replace('.', "/");
    let artifact_id = args[1];
    let version = args[2];
    let artifact_version = format!("{artifact_id}-{version}");
    Ok(format!(
        "{}{}/{}/{}/{}.jar",
        base_url, group_id, artifact_id, version, artifact_version
    ))
}

pub fn convert_to_full_path(base_path: String, library_name: &String) -> Result<String, AppError> {
    let args = library_name.split(':').collect::<Vec<_>>();
    if args.len() < 3 {
        return Err(AppError::InvalidPath(format!("Invalid library name: {}", library_name)));
    }
    let group_id = args[0].replace('.', "/");
    let artifact_id = args[1];
    let version = args[2];
    let artifact_version = format!("{artifact_id}-{version}");
    Ok(format!(
        "{}/{}/{}/{}/{}.jar",
        base_path, group_id, artifact_id, version, artifact_version
    ))
}

pub fn get_core_version(version_id: &String) -> Result<String, AppError> {
    let args = version_id.split('.').collect::<Vec<_>>();
    if args.len() < 2 {
        return Err(AppError::InvalidPath(format!("Invalid core version id: {}", version_id)));
    }
    Ok(format!("{}.{}", args[0], args[1]))
}

pub fn can_apply_rule(rule_obj: &Map<String, Value>) -> bool {
    let rules = match rule_obj.get("rules").and_then(|r| r.as_array()) {
        Some(r) => r,
        None => return true,
    };

    let mut allowed = false;

    for rule in rules {
        if let Some(rule_map) = rule.as_object() {
            let action = rule_map
                .get("action")
                .and_then(|a| a.as_str())
                .unwrap_or("allow");
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
        return current_os == name;
    }
    true
}

pub fn update_download_bar(progress: i64, app_handle: &AppHandle) {
    app_handle.emit("chek", progress)
        .unwrap_or_else(|x| info!("Failed to emit progress to progressBar event. detailed error: \n {x}"));
}

pub fn update_download_status(text: &str, app_handle: &AppHandle) {
    app_handle.emit("progress", text)
        .unwrap_or_else(|x| info!("Failed to emit text to progress event. detailed error: \n {x}"));
}

pub fn update_download(progress: i64, text: &str, app_handle: &AppHandle) {
    update_download_status(text, app_handle);
    update_download_bar(progress, app_handle);
}

pub fn patch_java_permission_linux(java: &Java) -> Result<(), AppError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let java_bin = java.get_bin_file();
        let metadata = std::fs::metadata(&java_bin).map_err(|e| AppError::FileNotFound(format!("Java binary not found at {:?}: {}", java_bin, e)))?;

        let mut permissions = metadata.permissions();
        let current_mode = permissions.mode();

        if current_mode & 0o111 == 0 {
            info!(
                "Adding execute permission to Java binary: {:?}",
                &java_bin
            );
            permissions.set_mode(current_mode | 0o111);
            std::fs::set_permissions(&java_bin, permissions)
                .map_err(|e| AppError::AccessDenied(format!("Failed to set execute permissions on Java binary: {}", e)))?;
        }
    }
    Ok(())
}

pub fn uuid_from_username(username: &str) -> Uuid {
    let input = format!("OfflinePlayer:{}", username);
    let hash = md5::compute(input.as_bytes());
    Builder::from_md5_bytes(*hash).into_uuid()
}

pub fn fetch_library_path(name: &String) -> Result<String, AppError> {
    let parts = name.split('/').collect::<Vec<&str>>();
    if parts.len() < 3 {
        return Err(AppError::InvalidPath(format!("Invalid library path format: {}", name)));
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    Ok(format!("{group}/{artifact}/{version}/{artifact}-{version}.jar"))
}

pub fn fetch_unofficial_library_repos(path: &String) -> Vec<String> {
    vec![
        format!("https://maven.minecraftforge.net/{path}"),
        format!("https://repo.spongepowered.org/maven/{path}"),
    ]
}

fn push_rule(_rules: &[Rule], pushing_vec: &mut Vec<String>, rule_os: &Option<RuleOS>) {
    if let Some(os) = rule_os {
        if let Some(name) = &os.name {
            pushing_vec.push(name.to_string());
        }
    } else {
        pushing_vec.push("osx".to_string());
        pushing_vec.push("windows".to_string());
        pushing_vec.push("linux".to_string());
    }
}

pub fn fetch_rules(value: Option<&Vec<Rule>>) -> LibraryRules {
    if let Some(rules) = value {
        let mut allowed = vec![];
        let mut disallowed = vec![];
        for rule in rules {
            let rule_action = &rule.action;
            let rule_os = &rule.os;
            if rule_action == "allow" {
                push_rule(rules, &mut allowed, rule_os);
            } else if rule_action == "disallow" {
                push_rule(rules, &mut disallowed, rule_os);
            }
        }
        LibraryRules {
            allowed_oses: allowed,
            disallowed_oses: disallowed,
        }
    } else {
        LibraryRules {
            allowed_oses: vec![
                "osx".to_string(),
                "windows".to_string(),
                "linux".to_string(),
            ],
            disallowed_oses: vec![],
        }
    }
}

pub fn is_legacy(version: &String) -> bool {
    if !version.starts_with("1.") {
        return false;
    }
    let mc_args = version.split('.').collect::<Vec<&str>>();
    if let Some(minor_version_str) = mc_args.get(1) {
        if let Ok(minor_version) = minor_version_str.parse::<u32>() {
            return minor_version <= 12;
        }
    }
    false
}

pub(crate) fn is_wayland() -> bool {
    let wayland_display = env::var("WAYLAND_DISPLAY").is_ok();
    let session_type = env::var("XDG_SESSION_TYPE")
        .map(|v| v.to_lowercase() == "wayland")
        .unwrap_or(false);

    wayland_display || session_type
}
pub fn create_reqwest_client(cfg: &Config) -> Result<ClientWithMiddleware, AppError> {
    let retry_policy = ExponentialBackoff::builder().build_with_total_retry_duration_and_max_retries(Duration::from_secs(1),3);

    let client =
    if cfg.download_settings.proxy != "" {
        let proxy = cfg.download_settings.proxy.clone();
        Client::builder().proxy(Proxy::all(proxy).map_err(|x| AppError::Reqwest(x))?).build().map_err(|x| AppError::Reqwest(x))
    } else {
        Client::builder().build().map_err(|x| AppError::Reqwest(x))
    };
    Ok(ClientBuilder::new(client?)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build())
}