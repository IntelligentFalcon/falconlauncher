use crate::models::downloader::{DownloadStage, PipelineProgressTracker};
use crate::models::error::AppError;
use crate::models::java::Java;
use crate::models::logger::LogLine;
use crate::models::mirror::Mirror;
use crate::models::platform::get_current_os_with_architecture;
use crate::services::directory_manager::get_java_dir;
use crate::services::game_downloader::download_file_if_not_exists;
use log::info;
use serde_json::Value;
use std::fs;
use std::fs::create_dir_all;
use std::time::Duration;
use tauri::State;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;
use crate::AppState;

pub fn get_java(java: String) -> Result<Java, AppError> {
    let runtime_dir = get_java_dir().join(&java);
    Java::new(runtime_dir)
}

pub async fn download_java(
    state: &State<'_,AppState>,
    java: &String,
    version: &String,
    _logger: &UnboundedSender<LogLine>,
    mirror: &Mirror,
    mut tracker: Option<&mut PipelineProgressTracker>,
    cancel_token: Option<&CancellationToken>,

) -> Result<(), AppError> {
    let runtime_dir = get_java_dir().join(java);
    let url = mirror.parse_url(&"https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json".to_string());
    let current_os = get_current_os_with_architecture();
    info!("Detected os and architecture is: {current_os}");
    let client = state.client.lock().await;
    let json = client.get(&url).timeout(Duration::from_secs(5)).send()
        .await
        .map_err(|e| AppError::NetworkRequestFailed(format!("Launcher Meta API request failed: {}", e)))?
        .json::<Value>()
        .await
        .map_err(|e| AppError::JsonParseFailed(format!("Failed to parse JDK results: {}", e)))?;

    let runtime_arr = json
        .get(&current_os)
        .and_then(|os_data| os_data.get(java))
        .and_then(|java_data| java_data.as_array())
        .ok_or_else(|| {
            AppError::ManifestParseFailed(format!(
                "Missing Java runtime data for OS {} and component {}",
                current_os, java
            ))
        })?;

    let runtime_v = runtime_arr
        .iter()
        .find(|x| {
            x.pointer("/version/name")
                .and_then(|name| name.as_str())
                .map(|s| s.to_lowercase().starts_with(version.as_str()))
                .unwrap_or(false)
        })
        .or_else(|| runtime_arr.first())
        .ok_or_else(|| AppError::ManifestParseFailed("Java runtime array is empty".to_string()))?;

    let runtime_manifest_url_str = runtime_v
        .pointer("/manifest/url")
        .and_then(|u| u.as_str())
        .ok_or_else(|| {
            AppError::ManifestParseFailed("Missing manifest URL for Java runtime".to_string())
        })?;

    let runtime_manifest_url = mirror.parse_url(&runtime_manifest_url_str.to_string());

    let runtime_manifest: Value = client.get(&runtime_manifest_url).timeout(Duration::from_secs(5)).send()
        .await
        .map_err(|e| AppError::NetworkRequestFailed(format!("Couldn't get runtime manifest: {}", e)))?
        .json()
        .await
        .map_err(|e| AppError::JsonParseFailed(format!("Failed to read the runtime json file: {}", e)))?;

    let files_map = runtime_manifest
        .get("files")
        .and_then(|f| f.as_object())
        .ok_or_else(|| {
            AppError::ManifestParseFailed(
                "Missing 'files' object in Java runtime manifest".to_string(),
            )
        })?;
    let total_download_files = files_map
        .values()
        .filter(|v| v.get("type").and_then(|t| t.as_str()) == Some("file"))
        .count();

    if let Some(t) = tracker.as_deref_mut() {
        t.start_stage(DownloadStage::Java, total_download_files);
    }

    for (k, v) in files_map {
        let file_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

        if file_type == "file" {
            let download_raw = v
                .pointer("/downloads/raw")
                .ok_or_else(|| AppError::ManifestParseFailed(format!("Missing raw downloads for {}", k)))?;

            let file_url_str = download_raw
                .get("url")
                .and_then(|u| u.as_str())
                .ok_or_else(|| AppError::ManifestParseFailed(format!("Missing URL for file {}", k)))?;

            let file_url = mirror.parse_url(&file_url_str.to_string());
            let size = download_raw.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
            let sha1 = download_raw.get("sha1").and_then(|s| s.as_str()).unwrap_or("");

            create_dir_all(&runtime_dir).map_err(|e| AppError::DirCreateFailed(e.to_string()))?;

            info!("Downloading {} ({} bytes)", file_url, size);
            download_file_if_not_exists(
                state,
                &runtime_dir.join(k),
                file_url.to_string(),
                sha1,
                size,
                tracker.as_deref(),
                cancel_token
            )
                .await?;

            if let Some(t) = tracker.as_deref_mut() {
                t.next_file();
            }
        } else {
            create_dir_all(runtime_dir.join(k))
                .map_err(|e| AppError::DirCreateFailed(format!("Failed to create directory {}: {}", k, e)))?;
        }
    }

    let release_file_path = runtime_dir.join("release");
    if !release_file_path.exists() {
        fs::write(
            &release_file_path,
            format!("JAVA_VERSION=\"{}\"", version),
        )
            .map_err(|x| AppError::FileCreateFailed(x.to_string()))?;
    }

    Ok(())
}