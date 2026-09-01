use crate::models::downloader::Manifest;
use crate::models::error::AppError::{FileReadFailed, Internal};
use crate::models::error::{AppError, Void};
use crate::models::mirror::Mirror;
use crate::models::versions::MinecraftVersion;
use crate::services::directory_manager::{get_versions_directory, version_manifest_directory};
use crate::services::game_downloader::download_file;
use crate::{AppState, GLOBAL_CACHE};
use log::debug;
use tauri::State;

/// Loads the version manifest, will download the file version manifest through the given mirror, if it doesn't exist
pub async fn load_version_manifest(state: &State<'_,AppState>) -> Result<Manifest, AppError> {
    if !version_manifest_directory().exists() {
        download_version_manifest(state).await?;
    }
    load_version_manifest_local()
}

/// Downloads the latest version manifest available through the given mirror whether it already exists or not.
/// returns Manifest itself if everything goes well or else an error will be dropped.
pub async fn refresh_version_manifest(state: &State<'_,AppState>) -> Result<Manifest, AppError> {
    download_version_manifest(state).await?;
    load_version_manifest_local()
}

/// Loads version manifest only from local files, will throw an error if it doesn't exist, its best to use load_version_manifest.
pub fn load_version_manifest_local() -> Result<Manifest, AppError> {
    let path = version_manifest_directory();
    let text = std::fs::read_to_string(&path).map_err(|x| FileReadFailed(x.to_string()))?;
    serde_json::from_str(text.as_str()).map_err(|x| AppError::JsonParseFailed(x.to_string()))
}

pub async fn load_installed_versions() {
    let mut versions = Vec::new();

    if let Ok(versions_dir) = get_versions_directory().read_dir() {
        for entry in versions_dir.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let path = entry.path();
                    if let Ok(mut sub_dir) = path.read_dir() {
                        let has_json = sub_dir.any(|entry_res| {
                            if let Ok(ent) = entry_res {
                                if let Some(name) = ent.file_name().to_str() {
                                    return name.to_lowercase().contains(".json");
                                }
                            }
                            false
                        });

                        if has_json {
                            if let Ok(mc_version) = MinecraftVersion::from_folder(path) {
                                versions.push(mc_version);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut global = GLOBAL_CACHE.lock().await;
    global.versions = versions;
}

pub async fn download_version_manifest(state: &State<'_, AppState>) -> Result<(), AppError> {
    let url = &state.config.read().await.download_settings.mirror
        .parse_url(&"https://launchermeta.mojang.com/mc/game/version_manifest.json".to_string());
    download_file(
        state,
        url.to_string(),
        &version_manifest_directory(),
        None,
        None
    )
        .await
}