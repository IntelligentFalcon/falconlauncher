use crate::utils::load_json_url;
use serde::Deserialize;
use serde_json::Value;

/// Returns the version_manifest.json file in a Value structure if the parse process was succeed.
pub async fn download_version_manifest() -> Manifest {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    let resp = reqwest::get(url).await.unwrap();
    serde_json::from_str(&resp.text().await.unwrap()).unwrap()
}

// pub async fn load_versions() -> Manifest {
//     let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
//     let resp = reqwest::get(url).await;
//     match resp {
//         Ok(r) => {}
//         Err(_) => {
//
//         }
//     }
// }

pub async fn load_version_manifest() -> Option<Value> {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    load_json_url(&url.to_string()).await
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
