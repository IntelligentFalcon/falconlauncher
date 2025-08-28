use crate::directory_manager::get_versions_directory;
use crate::downloader::{
    download_file, get_available_fabric_versions, get_available_forge_versions,
};
use crate::structs::VersionBase::{FABRIC, FORGE};
use crate::structs::{VersionBase, VersionCategory};
use crate::utils::load_json_url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::PartialEq;
use std::fmt::format;

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
    let snapshots: Vec<VersionLoader> = manifest
        .versions
        .iter()
        .filter(|x| x.version_type == VersionType::Snapshot)
        .map(|x| VersionLoader {
            id: x.id.to_string(),
            base: VersionBase::VANILLA,
            date: x.time.to_string(),
        })
        .collect();
    let old_beta: Vec<VersionLoader> = manifest
        .versions
        .iter()
        .filter(|x| x.version_type == VersionType::OldBeta)
        .map(|x| VersionLoader {
            id: x.id.to_string(),
            base: VersionBase::VANILLA,
            date: x.time.to_string(),
        })
        .collect();
    let old_alpha: Vec<VersionLoader> = manifest
        .versions
        .iter()
        .filter(|x| x.version_type == VersionType::OldAlpha)
        .map(|x| VersionLoader {
            id: x.id.to_string(),
            base: VersionBase::VANILLA,
            date: x.time.to_string(),
        })
        .collect();

    result.push(VersionCategory {
        name: "Snapshot".to_string(),
        versions: snapshots,
    });
    for ver in versions {
        let id = ver.id.clone();
        let id_args: Vec<&str> = id.split(".").collect();
        let category = format!("{}.{}", id_args[0], id_args[1]);
        if result.iter_mut().find(|x| x.name == category).is_none() {
            result.push(VersionCategory {
                name: category.clone(),
                versions: Vec::new(),
            });
        }
        let cat = result.iter_mut().find(|x| x.name == category).unwrap();
        cat.versions.push(VersionLoader {
            id: id.clone(),
            base: VersionBase::VANILLA,
            date: ver.release_time.clone(),
        });
        if forge {
            cat.versions.extend(
                get_available_forge_versions(&id)
                    .await
                    .iter()
                    .map(|x| VersionLoader {
                        id: x.clone(),
                        base: FORGE,
                        date: "FORGE".to_string(),
                    })
                    .collect::<Vec<_>>(),
            );
        }

        if fabric {
            cat.versions.extend(
                get_available_fabric_versions(&id)
                    .await
                    .iter()
                    .map(|x| VersionLoader {
                        id: x.clone(),
                        base: FABRIC,
                        date: "FABRIC".to_string(),
                    })
                    .collect::<Vec<_>>(),
            );
        }
    }
    result.push(VersionCategory {
        name: "Beta".to_string(),
        versions: old_beta,
    });
    result.push(VersionCategory {
        name: "Alpha".to_string(),
        versions: old_alpha,
    });

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
#[derive(Debug, Deserialize, Serialize)]
pub struct VersionLoader {
    pub id: String,
    pub base: VersionBase,
    pub date: String,
}

impl VersionLoader {
    pub fn get_fabric_loader_id(&self) -> String {
        self.id.split("-").collect::<Vec<&str>>()[1].to_string()
    }
    pub fn get_fabric_version_id(&self) -> String {
        self.id.split("-").collect::<Vec<&str>>()[0].to_string()
    }
}

impl VersionLoader {
    pub fn get_installed_id(&self) -> String {
        match self.base {
            VersionBase::VANILLA => self.id.clone(),
            FORGE => {
                let id_clone = self.id.clone();
                let args = id_clone.split("-").collect::<Vec<_>>();
                let vanilla_id = args[0];
                let forge_ver = args[1].split("-").last().unwrap();
                format!("{}-forge-{}", vanilla_id, forge_ver)
            }
            /// FIX THESE LATER
            VersionBase::NEOFORGE => self.id.clone(),
            FABRIC => {
                let args = self.id.split("-").collect::<Vec<_>>();
                format!("fabric-loader-{}-{}",args[1],args[0])
            }
            VersionBase::LITELOADER => self.id.clone(),
        }
    }
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
