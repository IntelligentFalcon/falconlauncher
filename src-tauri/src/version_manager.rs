use crate::directory_manager::get_versions_directory;
use crate::downloader::download_file_if_not_exists;
use crate::utils::load_json_url;
use serde::Deserialize;
use serde_json::Value;

pub async fn load_version_manifest() -> Option<Value> {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    load_json_url(&url.to_string()).await
}

pub async fn load_version_manifest_local() -> Option<Value> {
    let path = get_versions_directory().join("version_manifest_v2.json");
    let text = std::fs::read_to_string(&path);
    serde_json::from_str(text.unwrap().as_str()).expect("Failed to parse json")
}
pub async fn download_version_manifest() {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    download_file_if_not_exists(
        &get_versions_directory().join("version_manifest_v2.json"),
        url.to_string(),
        0,
    )
    .await;
}
#[derive(Deserialize, Debug)]
pub struct Manifest {
    latest: LatestVersionDetail,
    versions: Vec<VersionInfo>,
}
#[derive(Debug, Deserialize)]

pub struct LatestVersionDetail {
    pub release: String,
    pub snapshot: String,
}
#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub url: String,
    pub time: Option<String>,
    #[serde(rename = "releaseTime")]
    pub release_time: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
}
