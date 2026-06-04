pub mod commands;
pub mod models;
pub mod services;

use crate::commands::downloader::get_categorized_versions;
use crate::commands::mirrors::{get_available_mirrors, get_mirror, set_mirror};
use crate::commands::profiles::{create_offline_profile, get_profiles};
use crate::models::config::Config;
use crate::models::downloader::VersionLoader;
use crate::models::logger::{info, info_launcher, init_log_bridge, LogLine};
use crate::services::config::load;
use log::info;
use models::error::{Returns, Void};
use models::mirror::{mirror_from, mojang_mirror};
use models::mods::ModInfo;
use models::versions::MinecraftVersion;
use models::versions::VersionBase::{FABRIC, FORGE};
use services::directory_manager::{
    create_necessary_dirs, get_falcon_launcher_directory, get_mods_folder,
};
use services::downloader::{download_fabric, download_forge_version, GLOBAL_CACHE};
use services::game_launcher::{launch_game, update_download_status};
use services::mod_manager;
use services::mod_manager::{load_mods, set_mod_enabled};
use services::version_manager::{download_version_manifest, reload_installed_versions};
use services::{directory_manager, downloader};
use std::collections::VecDeque;
use std::env;
use std::fs::create_dir_all;
use std::string::ToString;
use std::sync::{Arc, Mutex};
use tauri::async_runtime::{block_on, spawn};
use tauri::{command, AppHandle, Manager, State};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_log::{Target, TargetKind, TimezoneStrategy};
use tokio::fs::copy;
use tokio::sync::{mpsc, RwLock};

pub struct FalconLauncher {
    pub name: String,
    pub version: String,
}
pub struct AppState {
    pub config: Arc<RwLock<Config>>,
    pub launcher_details: FalconLauncher,
    pub log_tx: mpsc::UnboundedSender<LogLine>,
    pub log_history: Arc<Mutex<VecDeque<LogLine>>>,
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {
            let _ = _app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Folder {
                        path: get_falcon_launcher_directory(),
                        file_name: Some("logs".to_string()),
                    }),
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                ])
                .timezone_strategy(TimezoneStrategy::UseLocal)
                .build(),
        )
        .setup(move |app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            info!("Linux user detected!!!");

            let fl_path = get_falcon_launcher_directory();
            let jdk_path = directory_manager::get_launcher_java_directory();
            let _ = create_dir_all(fl_path);
            let _ = create_dir_all(jdk_path);

            spawn(async {
                create_necessary_dirs().await;

                if mojang_mirror().is_connected().await {
                    download_version_manifest(&mojang_mirror()).await.unwrap();
                }
            });
            let app_handle = app.handle().clone();
            let shared_history = Arc::new(Mutex::new(VecDeque::with_capacity(1000)));

            // 2. Clone the Arc pointer for the bridge initialization
            let bridge_history = shared_history.clone();
            let log_tx = init_log_bridge(app_handle, bridge_history);
            app.manage(AppState {
                config: Arc::new(RwLock::new(load())),
                launcher_details: FalconLauncher {
                    name: "FalconLauncher".to_string(),
                    version: "BETA".to_string(),
                },
                log_tx,
                log_history: shared_history,
            });
            block_on(async {
                reload_installed_versions().await;
            });
            let window = app.handle().get_window("main").unwrap();

            window.center().expect("Failed to center the window");
            window.set_focus().expect("Failed to set window on focus");
            #[cfg(any(windows, target_os = "linux"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                app.deep_link().register("falconLauncher")?;
                app.deep_link().register_all()?;
            }
            app.deep_link().on_open_url(|event| {
                println!("deep link URLs: {:?}", event.urls());
            });
            return Ok(());
        })
        .invoke_handler(tauri::generate_handler![
            play,
            get_versions,
            commands::settings::get_maximum_ram_usage,
            commands::settings::get_minimum_ram_usage,
            commands::settings::set_maximum_ram_usage,
            commands::settings::set_minimum_ram_usage,
            commands::settings::set_language,
            commands::settings::get_language,
            commands::settings::set_exit_on_launch,
            commands::settings::should_exit_on_launch,
            commands::settings::save,
            set_config,
            get_username,
            set_username,
            get_total_ram,
            toggle_mod,
            delete_mod,
            get_mods,
            download_version,
            get_profiles,
            get_installed_versions,
            get_non_installed_versions,
            create_offline_profile,
            get_categorized_versions,
            commands::logger::get_log_history,
            commands::logger::clear_log_history_channel,
            commands::logger::clear_log_history,
            install_mod_from_local,
            get_available_mirrors,
            set_mirror,
            get_mirror,
            commands::mirrors::import_mirror,
            debug
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
#[command]
async fn toggle_mod(mod_info: ModInfo, toggle: bool) -> Void {
    set_mod_enabled(mod_info, toggle)
}
#[command]
async fn play(app: AppHandle, state: State<'_, AppState>, selected_version: String) -> Void {
    launch_game(app, selected_version, &*GLOBAL_CACHE.lock().await).await
}

/// Gives the available versions to download
#[command]
async fn get_versions() -> Returns<Vec<String>> {
    let global = GLOBAL_CACHE.lock().await;
    Ok(global
        .versions
        .iter()
        .map(|x| x.id.to_string())
        .clone()
        .collect())
}
#[command]
async fn get_mods() -> Returns<Vec<ModInfo>> {
    Ok(load_mods())
}
/// LINUX Debugger for the js side. use the developer console if you are on Windows build to check logs
#[command]
async fn debug(text: String) -> Void {
    println!("{}", text);
    Ok(())
}
#[command]
async fn get_total_ram() -> Returns<u64> {
    let ram = sys_info::mem_info().unwrap();
    Ok(ram.total)
}

#[command]
async fn set_config(state: State<'_, AppState>, config: Config) -> Void {
    let mut cfg = state.config.write().await;
    cfg.launch_options = config.launch_options;
    cfg.launcher_settings = config.launcher_settings;
    cfg.write_to_file();
    Ok(())
}

#[command]
async fn get_username(state: State<'_, AppState>) -> Returns<String> {
    let cfg = state.config.read().await;
    Ok(cfg.launch_options.username.clone())
}

#[command]
async fn set_username(state: State<'_, AppState>, username: String) -> Void {
    let mut cfg = state.config.write().await;
    cfg.launch_options.username = username;

    Ok(())
}

#[command]
async fn get_installed_versions() -> Returns<Vec<String>> {
    let global = GLOBAL_CACHE.lock().await;
    let versions = global.versions.clone();
    Ok(versions
        .iter()
        .filter(|x| x.is_installed())
        .map(|x| x.id.clone())
        .collect())
}
#[command]
async fn get_non_installed_versions() -> Returns<Vec<String>> {
    let global = GLOBAL_CACHE.lock().await;
    let versions = global.versions.clone();
    Ok(versions
        .iter()
        .filter(|x| !x.is_installed())
        .map(|x| x.id.clone())
        .collect())
}

#[command]
async fn install_mod_from_local(app: AppHandle) -> Void {
    let paths = app
        .dialog()
        .file()
        .add_filter("Minecraft mods".to_string(), &[&"jar", &"disabled"])
        .blocking_pick_files()
        .unwrap();
    for path in paths {
        let p = path.as_path().unwrap();
        let file_name = p.file_name().unwrap().to_str().unwrap();
        let new_path = get_mods_folder().join(file_name);
        copy(p, new_path).await.unwrap();
    }
    Ok(())
}
#[command]
async fn delete_mod(mod_info: ModInfo) -> Void {
    mod_manager::delete_mod(&mod_info);
    Ok(())
}
#[command]
async fn download_version(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    version_loader: VersionLoader,
) -> Void {
    let version_id = version_loader.get_installed_id();
    let cfg = &state.config.read().await;
    let mir = mirror_from(&cfg.download_settings.mirror);
    let logger = &state.log_tx;
    logger.send(info_launcher(format!(
        "DEBUG: Downloading version {} from 9craft mirror",
        version_loader.id
    )));
    if version_loader.base == FORGE {
        logger.send(info_launcher(format!(
            "DEBUG: Forge version detected! {} installing it rn!",
            version_loader.id
        )));
        download_forge_version(&version_loader.id, &app_handle, logger, &mir).await?;
    };
    if version_loader.base == FABRIC {
        logger.send(info_launcher(format!(
            "DEBUG: Fabric version detected! {} installing it rn!",
            version_loader.id
        )));
        download_fabric(&version_loader, logger, &mir).await?;
    }

    let version = MinecraftVersion::from_id(version_id);
    let inherited_version = version.get_inherited();
    update_download_status("Downloading version...", &app_handle);
    let cfg = &state.config.read().await;
    downloader::download_version(&version, &app_handle, logger, &*cfg).await?;
    if inherited_version.id != version.id {
        downloader::download_version(&inherited_version, &app_handle, logger, &*cfg).await?;
    }
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
