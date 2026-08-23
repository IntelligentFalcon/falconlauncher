use crate::models::error::{AppError, Void};
use crate::models::modrinth::{DependencyType, ModrinthMod, ModrinthSearchResult, ModrinthVersion};
use crate::services::directory_manager::get_mods_directory;
use crate::services::game_downloader::download_file_if_not_exists;
use log::info;
use serde::{Deserialize, Serialize};
use tauri::command;

#[command]
pub async fn search_for_modrinth_project(
    name: String,
    facets: String,
    index: String,
    offset: u64,
    limit: u64,
) -> Result<ModrinthSearchResult, AppError> {
    /// https://docs.modrinth.com/api/operations/searchprojects/ for more details
    let api = format!("https://api.modrinth.com/v2/search?query={name}&facets={facets}&offset={offset}&limit={limit}&index={index}");
    reqwest::get(&api)
        .await
        .map_err(|e| AppError::NetworkRequestFailed(format!("Modrinth API request failed: {}", e)))?
        .json::<ModrinthSearchResult>()
        .await
        .map_err(|e| {
            AppError::JsonParseFailed(format!("Failed to parse Modrinth search results: {}", e))
        })
}
#[command]
pub async fn get_modrinth_projects(project_id: String) -> Result<ModrinthMod, AppError> {
    /// https://docs.modrinth.com/api/operations/getproject/ for more details.
    let api = format!("https://api.modrinth.com/v2/project/{project_id}");
    reqwest::get(&api)
        .await
        .map_err(|e| AppError::NetworkRequestFailed(format!("Modrinth API request failed: {}", e)))?
        .json::<ModrinthMod>()
        .await
        .map_err(|e| AppError::JsonParseFailed(format!("Failed to parse Modrinth results: {}", e)))
}
#[command]
pub async fn list_modrinth_mod_versions(project_id: String) -> Result<Vec<ModrinthVersion>, AppError> {
    /// https://docs.modrinth.com/api/operations/getprojectversions/ for more details.
    let api = format!("https://api.modrinth.com/v2/project/{project_id}/version");
    reqwest::get(&api)
        .await
        .map_err(|e| AppError::NetworkRequestFailed(format!("Modrinth API request failed: {}", e)))?
        .json::<Vec<ModrinthVersion>>()
        .await
        .map_err(|e| AppError::JsonParseFailed(format!("Failed to parse Modrinth results: {}", e)))
}
#[command]
pub async fn get_modrinth_mod_dependencies(
    version: ModrinthVersion,
) -> Result<Vec<(ModrinthVersion, DependencyType)>, AppError> {
    let deps = version.dependencies;
    let mut result = Vec::new();
    for dep in deps.iter().filter(|x| x.version_id.is_some()) {
        let dep_type = dep.dependency_type.clone();
        let version = get_modrinth_mod_version_by_id(dep.clone().version_id.unwrap()).await?;
        result.push((version, dep_type))
    }
    Ok(result)
}
#[command]
pub async fn get_modrinth_mod_version_by_id(version_id: String) -> Result<ModrinthVersion, AppError> {
    let api = format!("https://api.modrinth.com/v2/version/{version_id}");
    reqwest::get(&api)
        .await
        .map_err(|e| AppError::NetworkRequestFailed(format!("Modrinth API request failed: {}", e)))?
        .json::<ModrinthVersion>()
        .await
        .map_err(|e| AppError::JsonParseFailed(format!("Failed to parse Modrinth results: {}", e)))
}
/// name: minecraft version's name
#[command]
pub async fn download_modrinth_mod_version(version: ModrinthVersion, name: String) -> Void {
    let files = version.files;
    let mods_dir = get_mods_directory();
    for file in files {
        let hashes = file.hashes;
        let sha1 = hashes.sha1.unwrap_or("".to_string());
        let size = file.size;
        let url = file.url;
        let full_path = mods_dir.join(file.file_name);
        info!("Downloading from {url} to {}", full_path.to_string_lossy());
        download_file_if_not_exists(&full_path, url, sha1.as_str(), size as u64, None).await?; // TODO: track progress
    }
    Ok(())
}
