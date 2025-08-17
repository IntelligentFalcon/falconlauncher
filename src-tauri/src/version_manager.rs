use crate::directory_manager::get_versions_directory;
use crate::downloader::download_file_if_not_exists;
use crate::structs::{MinecraftVersion, VersionCategory};
use crate::utils::{is_connected_to_internet, load_json_url};
use serde::Deserialize;
use serde_json::Value;
use std::cmp::PartialEq;
use std::ops::Add;

pub async fn load_version_manifest() -> Option<Value> {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    load_json_url(&url.to_string()).await
}

pub async fn load_version_manifest_local() -> Option<Manifest> {
    let path = get_versions_directory().join("version_manifest_v2.json");
    let text = std::fs::read_to_string(&path);
    serde_json::from_str(text.unwrap().as_str()).expect("Failed to parse json")
}

impl PartialEq for VersionType {
    fn eq(&self, other: &Self) -> bool {
        other == self
    }
}

pub async fn get_categorized_versions() -> Vec<VersionCategory> {
    if is_connected_to_internet().await {
        download_version_manifest().await;
    }

    let manifest = load_version_manifest_local()
        .await
        .expect("Failed to parse the manifest version");
    let mut result: Vec<VersionCategory> = Vec::new();
    /// ISSUE HERE
    let versions: Vec<&VersionInfo> = manifest
        .versions
        .iter()
        .filter(|x| x.version_type == VersionType::Release)
        .collect();
    /// END OF STACK OVERFLOW ISSUE
    for ver in versions {
        let id = ver.id.clone();
        println!("{}", ver.id);
        let id_args: Vec<&str> = id.split(".").collect();
        let category = format!("{}.{}", id_args[0], id_args[1]);
        let mut cat = result.iter().find(|&x| x.name == category);
        if cat.is_none() {
            result.push(VersionCategory {
                versions: vec![MinecraftVersion::from_id(ver.id.clone())],
                name: category,
            });
        } else {
            let mut unwrapped_cat = cat.unwrap();
            let mut versions = unwrapped_cat.versions.to_owned();
            versions.push(MinecraftVersion::from_id(ver.id.clone()));
        }
    }
    result
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
    pub latest: LatestVersionDetail,
    pub versions: Vec<VersionInfo>,
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
