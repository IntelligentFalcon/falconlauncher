use std::time::Duration;
use crate::models::error::{AppError, Void};
use crate::models::modrinth::{DependencyType, ModrinthMod, ModrinthSearchResult, ModrinthVersion};
use crate::services::directory_manager::get_mods_directory;
use crate::services::game_downloader::download_file_if_not_exists;
use log::info;
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, State};
use crate::{AppState, GLOBAL_CACHE};
use crate::models::downloader::{DownloadStage, PipelineProgressTracker};
use crate::models::error::AppError::Internal;
use crate::services::utils::create_reqwest_client;

#[command]
pub async fn search_for_modrinth_project(
    state: State<'_, AppState>,
    name: String,
    facets: String,
    index: String,
    offset: u64,
    limit: u64,
) -> Result<ModrinthSearchResult, AppError> {
    /// https://docs.modrinth.com/api/operations/searchprojects/ for more details
    let api = format!("https://api.modrinth.com/v2/search?query={name}&facets={facets}&offset={offset}&limit={limit}&index={index}");
    let client = state.client.lock().await;
    client.get(&api).timeout(Duration::from_secs(5)).send()
        .await
        .map_err(|e| AppError::NetworkRequestFailed(format!("Modrinth API request failed: {}", e)))?
        .json::<ModrinthSearchResult>()
        .await
        .map_err(|e| {
            AppError::JsonParseFailed(format!("Failed to parse Modrinth search results: {}", e))
        })
}
#[command]
pub async fn get_modrinth_projects(state: State<'_, AppState>,project_id: String) -> Result<ModrinthMod, AppError> {
    /// https://docs.modrinth.com/api/operations/getproject/ for more details.
    let api = format!("https://api.modrinth.com/v2/project/{project_id}");
    let client = state.client.lock().await;
    client.get(&api).timeout(Duration::from_secs(5)).send()
        .await
        .map_err(|e| AppError::NetworkRequestFailed(format!("Modrinth API request failed: {}", e)))?
        .json::<ModrinthMod>()
        .await
        .map_err(|e| AppError::JsonParseFailed(format!("Failed to parse Modrinth results: {}", e)))
}
#[command]
pub async fn list_modrinth_mod_versions(state: State<'_, AppState>, project_id: String) -> Result<Vec<ModrinthVersion>, AppError> {
    /// https://docs.modrinth.com/api/operations/getprojectversions/ for more details.
    let api = format!("https://api.modrinth.com/v2/project/{project_id}/version");
    let client = state.client.lock().await;
    client.get(&api).timeout(Duration::from_secs(5)).send()
        .await
        .map_err(|e| AppError::NetworkRequestFailed(format!("Modrinth API request failed: {}", e)))?
        .json::<Vec<ModrinthVersion>>()
        .await
        .map_err(|e| AppError::JsonParseFailed(format!("Failed to parse Modrinth results: {}", e)))
}
#[command]
pub async fn get_modrinth_mod_dependencies(
    state: State<'_, AppState>,
    version: ModrinthVersion,
) -> Result<Vec<(ModrinthVersion, DependencyType)>, AppError> {
    let deps = version.dependencies;
    let mut result = Vec::new();
    for dep in deps.iter().filter(|x| x.version_id.is_some()) {
        let dep_type = dep.dependency_type.clone();
        let version = _get_modrinth_mod_version_by_id(&state,dep.clone().version_id.unwrap_or("Unnamed Dependency".to_string())).await?;
        result.push((version, dep_type))
    }
    Ok(result)
}
#[command]
pub async fn get_modrinth_mod_version_by_id(state: State<'_, AppState>, version_id: String) -> Result<ModrinthVersion, AppError> {
    _get_modrinth_mod_version_by_id(&state, version_id).await
}

pub async fn _get_modrinth_mod_version_by_id(state: &State<'_, AppState>, version_id: String) -> Result<ModrinthVersion, AppError> {
    let api = format!("https://api.modrinth.com/v2/version/{version_id}");
    let client = state.client.lock().await;
    client.get(&api).send()
        .await
        .map_err(|e| AppError::NetworkRequestFailed(format!("Modrinth API request failed: {}", e)))?
        .json::<ModrinthVersion>()
        .await
        .map_err(|e| AppError::JsonParseFailed(format!("Failed to parse Modrinth results: {}", e)))
}

/// name: minecraft version's name
#[command]
pub async fn download_modrinth_mod_version(state: State<'_, AppState>,app_handle: AppHandle,version: ModrinthVersion, name: String) -> Void {
    let files = version.files;
    let mods_dir = get_mods_directory();
    for file in files {
        let hashes = file.hashes;
        let sha1 = hashes.sha1.unwrap_or("".to_string());
        let size = file.size;
        let url = file.url;
        let full_path = mods_dir.join(file.file_name);
        info!("Downloading from {url} to {}", full_path.to_string_lossy());
        let mut stages = Vec::new();
        stages.push((DownloadStage::Mod, 100f32));
        let pipe = PipelineProgressTracker::new(app_handle.clone(), &stages);
        download_file_if_not_exists(&state,&full_path, url, sha1.as_str(), size as u64, Some(&pipe)).await?;
    }
    Ok(())
}
