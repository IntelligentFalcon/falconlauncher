use crate::directory_manager::get_versions_directory;
use crate::downloader::{download_file, download_file_if_not_exists, get_available_forge_versions};
use crate::structs::{MinecraftVersion, VersionCategory};
use crate::utils::{extend_once, is_connected_to_internet, load_json_url};
use serde::{Deserialize, Serialize};
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

// impl PartialEq for VersionType {
//     fn eq(&self, other: &Self) -> bool {
//         other == self
//     }
// }

pub async fn get_categorized_versions(
    fabric: bool,
    forge: bool,
    neo_forge: bool,
    lite_loader: bool,
) -> Vec<VersionCategory> {
    let manifest = load_version_manifest_local()
        .await
        .expect("Failed to parse the manifest version");
    let mut result: Vec<VersionCategory> = Vec::new();
    let versions: Vec<&VersionInfo> = manifest
        .versions
        .iter()
        .filter(|x| x.version_type == VersionType::Release)
        .collect();
    for ver in versions {
        let id = ver.id.clone();
        let id_args: Vec<&str> = id.split(".").collect();
        let category = format!("{}.{}", id_args[0], id_args[1]);
        if let Some(cat) = result.iter_mut().find(|x| x.name == category) {
            cat.versions.push(MinecraftVersion::from_id(ver.id.clone()));
            if forge {
                cat.versions.extend(
                    get_available_forge_versions(&id)
                        .await
                        .iter()
                        .map(|x| MinecraftVersion::from_id(x.clone()))
                        .collect::<Vec<_>>(),
                );
            }
        } else {
            let mut v = vec![MinecraftVersion::from_id(id.clone())];
            if forge {
                v.extend(
                    get_available_forge_versions(&id)
                        .await
                        .iter()
                        .map(|x| MinecraftVersion::from_id(x.clone()))
                        .collect::<Vec<_>>(),
                );
            }
            result.push(VersionCategory {
                name: category,
                versions: v,
            });
        }
    }

    result
}
pub async fn download_version_manifest() {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    download_file(
        url.to_string(),
        get_versions_directory()
            .join("version_manifest_v2.json")
            .to_str()
            .unwrap()
            .to_string(),
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
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(PartialEq)]
pub enum VersionType {
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
}
