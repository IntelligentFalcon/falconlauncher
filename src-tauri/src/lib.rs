pub mod commands;
pub mod models;
pub mod services;

use crate::models::config::Config;
use crate::models::fabric::{FabricInstaller, FabricLoader, FabricMinecraftVersion};
use crate::models::logger::{init_log_bridge, LogLine};
use crate::services::config::load;
use models::error::{Returns, Void};
use models::mirror::mojang_mirror;
use models::mods::ModInfo;
use models::versions::MinecraftVersion;
use services::directory_manager::{
    create_necessary_dirs, get_falcon_launcher_directory, get_mods_folder,
};
use services::game_launcher::launch_game;
use services::mod_manager;
use services::mod_manager::{load_mods, set_mod_enabled};
use services::version_manager::{download_version_manifest, reload_installed_versions};
use std::collections::{HashMap, VecDeque};
use std::env;
use std::string::ToString;
use std::sync::{Arc, LazyLock, Mutex};
use log::info;
use tauri::async_runtime::{block_on, spawn};
use tauri::{command, AppHandle, Manager, State};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_log::{Target, TargetKind, TimezoneStrategy};
use tokio::fs::copy;
use tokio::sync;
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

pub struct Global {
    pub forge: Option<HashMap<String, Vec<String>>>,
    pub fabric_loaders: Option<Vec<FabricLoader>>,
    pub fabric_installers: Option<Vec<FabricInstaller>>,
    pub fabric_mc_versions: Option<Vec<FabricMinecraftVersion>>,
    pub versions: Vec<MinecraftVersion>,
}

pub static GLOBAL_CACHE: LazyLock<sync::Mutex<Global>> = LazyLock::new(|| {
    sync::Mutex::new(Global {
        forge: None,
        fabric_loaders: None,
        fabric_installers: None,
        fabric_mc_versions: None,
        versions: Vec::new(),
    })
});

pub const DEV_MODE: bool = false;
pub const LAUNCHER_NAME: &str = "FalconLauncher";
pub const LAUNCHER_VERSION: &str = "BETA-0.1";
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
                        file_name: Some("latest".to_string()),
                    }),
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                ])
                .timezone_strategy(TimezoneStrategy::UseLocal)
                .build(),
        )
        .setup(move |app| {
            info!("Launcher's initialization has started...");
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                if DEV_MODE {
                    window.open_devtools();
                }
            }

            info!("Successfully passed the debug assertions.");
            spawn(async {
                create_necessary_dirs().await;

                if mojang_mirror().is_connected().await {
                    download_version_manifest(&mojang_mirror()).await.unwrap();
                }
            });
            info!("Created required necessary directories.");
            let app_handle = app.handle().clone();
            let shared_history = Arc::new(Mutex::new(VecDeque::with_capacity(10000)));

            let bridge_history = shared_history.clone();
            let log_tx = init_log_bridge(app_handle, bridge_history);
            app.manage(AppState {
                config: Arc::new(RwLock::new(load())),
                launcher_details: FalconLauncher {
                    name: LAUNCHER_NAME.to_string(),
                    version: LAUNCHER_VERSION.to_string(),
                },
                log_tx,
                log_history: shared_history,
            });
            block_on(async {
                reload_installed_versions().await;
            });
            info!("Reloaded installed versions.");

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
                info!("deep link URLs: {:?}", event.urls());
            });
            info!("Program window's properties was modified successfully .");

            return Ok(());

        })
        .invoke_handler(tauri::generate_handler![
            play,
            commands::downloader::get_versions,
            commands::settings::get_maximum_ram_usage,
            commands::settings::get_minimum_ram_usage,
            commands::settings::set_maximum_ram_usage,
            commands::settings::set_minimum_ram_usage,
            commands::settings::set_language,
            commands::settings::get_language,
            commands::settings::set_exit_on_launch,
            commands::settings::should_exit_on_launch,
            commands::settings::save,
            commands::settings::set_config,
            commands::settings::get_username,
            commands::settings::set_username,
            commands::settings::get_total_ram,
            toggle_mod,
            delete_mod,
            get_mods,
            install_mod_from_local,
            commands::downloader::download_version,
            commands::downloader::get_installed_versions,
            commands::downloader::get_non_installed_versions,
            commands::downloader::get_forge_versions,
            commands::downloader::get_fabric_versions,
            commands::profiles::get_profiles,
            commands::profiles::create_offline_profile,
            commands::logger::get_log_history,
            commands::logger::clear_log_history_channel,
            commands::logger::clear_log_history,
            commands::logger::debug,
            commands::mirrors::get_available_mirrors,
            commands::mirrors::set_mirror,
            commands::mirrors::get_mirror,
            commands::mirrors::import_mirror
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

#[command]
async fn get_mods() -> Returns<Vec<ModInfo>> {
    Ok(load_mods())
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

