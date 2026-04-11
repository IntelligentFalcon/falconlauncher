use tauri::{command, AppHandle, Manager};
use tokio::io::AsyncReadExt;
use crate::AppState;
use crate::models::downloader::{VersionInfo, VersionLoader};
use crate::models::error::Returns;
use crate::models::mirror::{mirror, mirror_from, Mirror};
use crate::models::versions::{VersionBase, VersionCategory, VersionType};
use crate::models::versions::VersionBase::{FABRIC, FORGE};
use crate::services::downloader::{get_available_fabric_versions, get_available_forge_versions};
use crate::services::version_manager;

#[command]
pub async fn get_categorized_versions(
    app_handle: AppHandle,
    fabric: bool,
    forge: bool,
    neo_forge: bool,
    lite_loader: bool,
) -> Returns<Vec<VersionCategory>> {
    let state = app_handle.state::<AppState>();
    let cfg = state.config.read().await;
    let mirror = mirror_from(&cfg.download_settings.mirror);
    let manifest = version_manager::load_version_manifest(&mirror).await.expect("Failed to parse version manifest");
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

    Ok(result)
}