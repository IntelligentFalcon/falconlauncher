use crate::models::downloader::{Manifest, VersionInfo, VersionLoader};
use crate::models::error::{AppError, Void};
use crate::models::mirror::Mirror;
use crate::models::versions::VersionBase::{FABRIC, FORGE};
use crate::models::versions::{
    MinecraftVersion, VersionBase, VersionCategory, VersionNameBase, VersionType,
};
use crate::services::game_downloader;
use crate::services::game_downloader::{
    download_fabric, download_forge_version, download_from_manifest, get_available_fabric_versions,
    get_available_forge_versions,
};
use crate::services::utils::update_download_status;
use crate::services::version_manager::{download_version_manifest, load_version_manifest};
use crate::{AppState, GLOBAL_CACHE};
use log::info;
use std::thread::JoinHandle;
use tauri::{command, AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tokio_util::sync::CancellationToken;

const TOKEN_NAME: &str = "download_version";
#[command]
pub async fn get_vanilla_versions(
    _app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<VersionCategory>, AppError> {
    let cfg = state.config.read().await;
    let mirror = &cfg.download_settings.mirror;
    let mut result: Vec<VersionCategory> = Vec::new();

    let manifest = load_version_manifest(&state).await?;
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
        let id_args: Vec<&str> = id.split('.').collect();
        if id_args.len() < 2 {
            continue;
        }
        let category = format!("{}.{}", id_args[0], id_args[1]);

        if !result.iter().any(|x| x.name == category) {
            result.push(VersionCategory {
                name: category.clone(),
                versions: Vec::new(),
            });
        }

        if let Some(cat) = result.iter_mut().find(|x| x.name == category) {
            cat.versions.push(VersionLoader {
                id: id.clone(),
                base: VersionBase::VANILLA,
                date: ver.release_time.clone(),
            });
        }
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
    _app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<VersionCategory>, AppError> {
    let cfg = state.config.read().await;
    let mirror = &cfg.download_settings.mirror;

    let manifest = load_version_manifest(&state).await?;
    let mut result: Vec<VersionCategory> = Vec::new();
    let versions: Vec<&VersionInfo> = manifest
        .versions
        .iter()
        .filter(|x| matches!(x.version_type, VersionType::Release))
        .collect();

    for ver in versions {
        let id = ver.id.clone();
        let id_args: Vec<&str> = id.split('.').collect();
        if id_args.len() < 2 {
            continue;
        }
        let category = format!("{}.{}", id_args[0], id_args[1]);

        let pos = result.iter().position(|x| x.name == category);
        let cat = match pos {
            Some(idx) => &mut result[idx],
            None => {
                let c = VersionCategory {
                    versions: vec![],
                    name: category.clone(),
                };
                result.push(c);
                result.last_mut().ok_or_else(|| {
                    AppError::Internal(
                        "Vector state corrupted during category creation".to_string(),
                    )
                })?
            }
        };

        let mut forge_versions = get_available_forge_versions(&id, &state).await?;
        forge_versions.reverse();
        cat.versions
            .extend(forge_versions.into_iter().map(|x| VersionLoader {
                id: x,
                base: FORGE,
                date: "FORGE".to_string(),
            }));
    }

    Ok(result)
}

#[command]
pub async fn get_fabric_versions(
    _app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<VersionCategory>, AppError> {
    let manifest = load_version_manifest(&state).await?;

    let mut result: Vec<VersionCategory> = Vec::new();
    let versions: Vec<&VersionInfo> = manifest
        .versions
        .iter()
        .filter(|x| matches!(x.version_type, VersionType::Release))
        .collect();

    for ver in versions {
        let id = ver.id.clone();
        let id_args: Vec<&str> = id.split('.').collect();
        if id_args.len() < 2 {
            continue;
        }
        let category = format!("{}.{}", id_args[0], id_args[1]);

        let pos = result.iter().position(|x| x.name == category);
        let cat = match pos {
            Some(idx) => &mut result[idx],
            None => {
                let c = VersionCategory {
                    versions: vec![],
                    name: category.clone(),
                };
                result.push(c);
                result.last_mut().ok_or_else(|| {
                    AppError::Internal(
                        "Vector state corrupted during category creation".to_string(),
                    )
                })?
            }
        };

        let fabric_versions = get_available_fabric_versions(&state, &id).await?;

        cat.versions
            .extend(fabric_versions.into_iter().map(|x| VersionLoader {
                id: x,
                base: FABRIC,
                date: "FABRIC".to_string(),
            }));
    }

    Ok(result)
}

#[tauri::command]
pub async fn cancel_download(state: State<'_, AppState>) -> Result<(), AppError> {
    let token = state.download_manager.cancellation_token.lock().await;

    if let Some(token) = token.as_ref() {
        token.cancel();
    }

    Ok(())
}

#[command]
pub async fn download_version(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    version_loader: VersionLoader,
    name: String,
) -> Result<(), AppError> {
    let token = CancellationToken::new();
    info!("Started downloading process");
    {
        let mut current_token = state.download_manager.cancellation_token.lock().await;

        if let Some(old_token) = current_token.take() {
            old_token.cancel();
        }

        *current_token = Some(token.clone());
    }

    download_task(&app_handle, &state, &version_loader, name, Some(&token)).await?;

    Ok(())
}

pub async fn download_task(
    app_handle: &AppHandle,
    state: &State<'_, AppState>,
    version_loader: &VersionLoader,
    name: String,
    token: Option<&CancellationToken>,
) -> Result<(), AppError> {
    let mut version_id = version_loader.get_installed_id();
    let cfg = state.config.read().await;
    let mir = &cfg.download_settings.mirror;
    let logger = &state.log_tx;

    info!(
        "DEBUG: Downloading version {} from {} mirror",
        version_loader.id, mir.name
    );

    if version_loader.base == FORGE {
        info!(
            "DEBUG: Forge version detected! {} installing now!",
            version_loader.id
        );
        let t = download_forge_version(
            &state,
            &version_loader.id,
            &app_handle,
            logger,
            mir,
            &mut version_id,
            None,
            token,
        )
        .await;

        if let Err(e) = &t {
            info!("Forge installation error: {:?}", e);
        }
    }

    if version_loader.base == FABRIC {
        info!(
            "DEBUG: Fabric version detected! {} installing now!",
            version_loader.id
        );
        download_fabric(&state, &version_loader, logger, mir, None, token).await?;
    }

    info!("Downloading {version_id}.json");

    let manifest = load_version_manifest(&state).await?;
    if version_loader.base == VersionBase::VANILLA {
        download_from_manifest(&state, &version_id, &manifest, mir, None, token).await?;
    }

    let version = MinecraftVersion::from_id(version_id);
    let inherited_version = version.get_inherited();
    info!("Detected inherited version is {}", inherited_version.id);

    update_download_status("Downloading version...", &app_handle);

    let downloadable_version = if version_loader.base == VersionBase::VANILLA {
        &version
    } else {
        &inherited_version
    };

    game_downloader::download_version(
        &state,
        downloadable_version,
        &name,
        &app_handle,
        logger,
        token,
    )
    .await?;

    if inherited_version.id != version.id {
        game_downloader::download_version(&state, &version, &name, &app_handle, logger, token)
            .await?;
    }

    update_download_status("", &app_handle);

    app_handle
        .dialog()
        .message("Successfully installed the selected version! You can now play it.")
        .title("Done!")
        .blocking_show();

    let mut global = GLOBAL_CACHE.lock().await;
    if !global.versions.iter().any(|x| x.id == version.id) {
        global.versions.push(version);
    }
    Ok(())
}

#[command]
pub async fn get_versions() -> Result<Vec<VersionNameBase>, AppError> {
    let global = GLOBAL_CACHE.lock().await;

    Ok(global
        .versions
        .iter()
        .map(|x| VersionNameBase {
            name: x.id.to_string(),
            base: x
                .load_json()
                .get("inheritsFrom")
                .map(|v| v.as_str().unwrap_or(x.id.as_str()))
                .unwrap_or(x.id.as_str())
                .to_string(),
            loader: if x.id.to_lowercase().contains("fabric") {
                "fabric".to_string()
            } else if x.id.to_lowercase().contains("forge") {
                "forge".to_string()
            } else {
                "vanilla".to_string()
            },
        })
        .collect())
}

#[command]
pub async fn get_installed_versions() -> Result<Vec<String>, AppError> {
    let global = GLOBAL_CACHE.lock().await;
    Ok(global
        .versions
        .iter()
        .filter(|x| x.is_installed())
        .map(|x| x.id.clone())
        .collect())
}

#[command]
pub async fn reload_version_manifest(_app_handle: AppHandle, state: State<'_, AppState>) -> Void {
    let cfg = state.config.read().await;
    let mirror = &cfg.download_settings.mirror;
    download_version_manifest(&state).await
}
