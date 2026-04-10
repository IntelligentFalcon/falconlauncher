use crate::models::downloader::{Manifest, VersionInfo, VersionLoader};
use crate::services::directory_manager::{get_versions_directory, version_manifest_directory};
use crate::services::downloader::{download_file, get_available_fabric_versions, get_available_forge_versions, GLOBAL_CACHE};
use crate::models::mirror::Mirror;
use crate::models::error::Void;
use crate::models::versions::VersionBase::{FABRIC, FORGE};
use crate::models::versions::{MinecraftVersion, VersionBase, VersionCategory, VersionType};

pub async fn load_version_manifest(mirror: &Mirror) -> Option<Manifest> {
    // let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    download_version_manifest(mirror).await;
    load_version_manifest_local()
}

pub fn load_version_manifest_local() -> Option<Manifest> {
    let path = version_manifest_directory();
    let text = std::fs::read_to_string(&path);
    serde_json::from_str(text.unwrap().as_str()).expect("Failed to parse json")
}
pub async fn reload_installed_versions() {
    let versions_dir = get_versions_directory().read_dir().unwrap();
    let versions = versions_dir.filter_map(|x| {
        let d = x.unwrap();
        if d.file_type().unwrap().is_file() {
            return None;
        }

        if d.path().read_dir().unwrap().find(|x| {
            let ent = x.as_ref().unwrap();
            ent.file_name().to_str().unwrap().to_lowercase().contains(".json")
        }).is_some() {
            return  Some(MinecraftVersion::from_folder(d.path()));
        }
        return None;
    }).collect::<Vec<MinecraftVersion>>();
    let mut global = GLOBAL_CACHE.lock().await;
    global.versions = versions;

}

pub async fn get_categorized_versions(
    fabric: bool,
    forge: bool,
    neo_forge: bool,
    lite_loader: bool,
) -> Vec<VersionCategory> {
    let manifest = load_version_manifest_local().expect("Failed to parse the manifest version");
    let mut result: Vec<VersionCategory> = Vec::new();
    let versions: Vec<&VersionInfo> = manifest
        .versions
        .iter().filter(|x| matches!(x.version_type, VersionType::Release))
        .collect();
    let snapshots: Vec<VersionLoader> = manifest
        .versions
        .iter()
        .filter(|x| matches!(x.version_type, VersionType::Snapshot))
        .map(|x| VersionLoader {
            id: x.id.to_string(),
            base: VersionBase::VANILLA,
            date: x.time.to_string(),
        })
        .collect();
    let old_beta: Vec<VersionLoader> = manifest
        .versions
        .iter()
        .filter(|x| matches!(x.version_type, VersionType::OldBeta))
        .map(|x| VersionLoader {
            id: x.id.to_string(),
            base: VersionBase::VANILLA,
            date: x.time.to_string(),
        })
        .collect();
    let old_alpha: Vec<VersionLoader> = manifest
        .versions
        .iter()
        .filter(|x| matches!(x.version_type, VersionType::OldAlpha))
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
pub async fn initialize_versions(){
    let mut global = GLOBAL_CACHE.lock().await;
    let manifest = load_version_manifest_local().expect("Failed to parse the manifest version");
    for v in &manifest.versions {
        global.versions.push(MinecraftVersion::from_id(v.id.clone()));
    }
}
pub async fn download_version_manifest(mirror: &Mirror) -> Void {
    let url = mirror.parse_url(&"https://launchermeta.mojang.com/mc/game/version_manifest.json".to_string());
    download_file(
        url.to_string(),
        version_manifest_directory().to_str()
            .unwrap()
            .to_string(),
    )
    .await
}

