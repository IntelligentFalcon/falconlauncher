use log::info;
use crate::models::downloader::{VersionInfo, VersionLoader};
use crate::models::error::AppError;
use crate::models::versions::VersionBase::{FABRIC, FORGE};
use crate::models::versions::{MinecraftVersion, VersionBase, VersionCategory, VersionType};
use crate::services::game_downloader::{
    download_fabric, download_forge_version, get_available_fabric_versions,
    get_available_forge_versions,
};
use crate::services::utils::update_download_status;
use crate::services::{game_downloader, version_manager};
use crate::{AppState, GLOBAL_CACHE};
use tauri::{command, AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use crate::services::version_manager::load_version_manifest;

#[command]
pub async fn get_vanilla_versions(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<VersionCategory>, AppError> {
    let manifest = version_manager::load_version_manifest_local().map_err(|x| {
        AppError::ManifestParseFailed("Failed to parse version manifest".to_string())
    })?;
    let cfg = state.config.read().await;
    let mut result: Vec<VersionCategory> = Vec::new();
    let versions: Vec<&VersionInfo> = manifest
        .versions
        .iter()
        .filter(|x| matches!(x.version_type, VersionType::Release))
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
    }
    result.push(VersionCategory {
        name: "Snapshot".to_string(),
        versions: snapshots,
    });
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

#[command]
pub async fn get_forge_versions(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<VersionCategory>, AppError> {
    let manifest = version_manager::load_version_manifest_local().map_err(|x| {
        AppError::ManifestParseFailed("Failed to parse version manifest".to_string())
    })?;
    let cfg = state.config.read().await;
    let mirror = &cfg.download_settings.mirror;
    let mut result: Vec<VersionCategory> = Vec::new();
    let versions: Vec<&VersionInfo> = manifest
        .versions
        .iter()
        .filter(|x| matches!(x.version_type, VersionType::Release))
        .collect();
    for ver in versions {
        let id = ver.id.clone();
        let id_args: Vec<&str> = id.split(".").collect();
        let category = format!("{}.{}", id_args[0], id_args[1]);
        let cat_opt = result.iter_mut().find(|x| x.name == category.clone());
        let cat = if cat_opt.is_none() {
            let c = VersionCategory {
                versions: vec![],
                name: category.clone(),
            };
            result.push(c);
            result.iter_mut().find(|x| x.name == category).unwrap()
        } else {
            cat_opt.unwrap()
        };
        cat.versions.extend(
            get_available_forge_versions(&id, &mirror)
                .await?
                .iter()
                .map(|x| VersionLoader {
                    id: x.clone(),
                    base: FORGE,
                    date: "FORGE".to_string(),
                })
                .collect::<Vec<_>>(),
        );
    }
    Ok(result)
}

#[command]
pub async fn get_fabric_versions(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<VersionCategory>, AppError> {
    let manifest = version_manager::load_version_manifest_local().map_err(|x| {
        AppError::ManifestParseFailed("Failed to parse version manifest".to_string())
    })?;
    let cfg = state.config.read().await;
    let mirror = &cfg.download_settings.mirror;
    let mut result: Vec<VersionCategory> = Vec::new();
    let versions: Vec<&VersionInfo> = manifest
        .versions
        .iter()
        .filter(|x| matches!(x.version_type, VersionType::Release))
        .collect();
    for ver in versions {
        let id = ver.id.clone();
        let id_args: Vec<&str> = id.split(".").collect();
        let category = format!("{}.{}", id_args[0], id_args[1]);
        let cat_opt = result.iter_mut().find(|x| x.name == category.clone());
        let cat = if cat_opt.is_none() {
            let c = VersionCategory {
                versions: vec![],
                name: category.clone(),
            };
            result.push(c);
            result.iter_mut().find(|x| x.name == category).unwrap()
        } else {
            cat_opt.unwrap()
        };

        cat.versions.extend(
            get_available_fabric_versions(&id)
                .await?
                .iter()
                .map(|x| VersionLoader {
                    id: x.clone(),
                    base: FABRIC,
                    date: "FABRIC".to_string(),
                })
                .collect::<Vec<_>>(),
        );
    }
    Ok(result)
}
#[command]
pub async fn download_version(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    version_loader: VersionLoader,
    name: String,
) -> Result<(), AppError> {
    let mut version_id = version_loader.get_installed_id();
    let cfg = &state.config.read().await;
    let mir = &cfg.download_settings.mirror;
    let logger = &state.log_tx;
    info!(
        "DEBUG: Downloading version {} from {} mirror",
        version_loader.id,
        mir.name
    );
    if version_loader.base == FORGE {
        info!(
            "DEBUG: Forge version detected! {} installing it rn!",
            version_loader.id
        );
        let t = download_forge_version(
            &version_loader.id,
            &app_handle,
            logger,
            &mir,
            &mut version_id,
        )
        .await;
        if let Err(e) = &t {
            info!("{:?}", e);
        }
    };
    if version_loader.base == FABRIC {
    info!(
            "DEBUG: Fabric version detected! {} installing it rn!",
            version_loader.id
        );
        download_fabric(&version_loader, logger, &mir).await?;
    }

    info!("Downloading {version_id}.json");

    let manifest = load_version_manifest(mir).await?;
    game_downloader::download_from_manifest(&version_id, &manifest, mir)
        .await?;
    let version = MinecraftVersion::from_id(version_id);

    let inherited_version = version.get_inherited();
    update_download_status("Downloading version...", &app_handle);
    let cfg = &state.config.read().await;
    if inherited_version.id != version.id {
        game_downloader::download_version(&inherited_version, &name, &app_handle, logger, &*cfg)
            .await?;
    }
    game_downloader::download_version(&version, &name, &app_handle, logger, &*cfg).await?;
    update_download_status("", &app_handle);
    app_handle
        .dialog()
        .message("Successfully installed the selected version you can now play it")
        .title("Done!")
        .blocking_show();
    let mut global = GLOBAL_CACHE.lock().await;
    global.versions.push(version);
    Ok(())
}

/// Gives the available versions to download
#[command]
pub async fn get_versions() -> Result<Vec<String>, AppError> {
    let global = GLOBAL_CACHE.lock().await;
    Ok(global
        .versions
        .iter()
        .map(|x| x.id.to_string())
        .clone()
        .collect())
}

#[command]
pub async fn get_non_installed_versions() -> Result<Vec<String>, AppError> {
    let global = GLOBAL_CACHE.lock().await;
    let versions = global.versions.clone();
    Ok(versions
        .iter()
        .filter(|x| !x.is_installed())
        .map(|x| x.id.clone())
        .collect())
}

#[command]
pub async fn get_installed_versions() -> Result<Vec<String>, AppError> {
    let global = GLOBAL_CACHE.lock().await;
    let versions = global.versions.clone();
    Ok(versions
        .iter()
        .filter(|x| x.is_installed())
        .map(|x| x.id.clone())
        .collect())
}
