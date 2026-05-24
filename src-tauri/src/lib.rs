pub mod commands;
pub mod models;
pub mod services;

use crate::commands::downloader::get_categorized_versions;
use crate::commands::profiles::{create_offline_profile, get_profiles};
use crate::models::config::Config;
use crate::models::downloader::VersionLoader;
use crate::services::config::load;
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
use services::utils::is_connected_to_internet;
use services::version_manager::{
    download_version_manifest, reload_installed_versions,
};
use services::{directory_manager, downloader};
use std::env;
use std::fs::create_dir_all;
use std::string::ToString;
use std::sync::Arc;
use tauri::async_runtime::{block_on, spawn};
use tauri::{command, AppHandle, Manager, State};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_log::{Target, TargetKind, TimezoneStrategy};
use tokio::fs::copy;
use tokio::sync::RwLock;
use tracing::info;

pub struct FalconLauncher {
    pub name: String,
    pub version: String
}
pub struct AppState {
    pub config: Arc<RwLock<Config>>,
    pub launcher_details: FalconLauncher,
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
        .plugin(tauri_plugin_log::Builder::new().timezone_strategy(TimezoneStrategy::UseLocal).build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(Target::new(TargetKind::Folder {
                    path: get_falcon_launcher_directory(),
                    file_name: Some("logs".to_string()),
                }))
                .build(),
        )
        .setup(move |app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                // window.open_devtools();
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
            app.manage(AppState {
                config: Arc::new(RwLock::new(load())),
                launcher_details: FalconLauncher {
                    name: "FalconLauncher".to_string(),
                    version: "BETA".to_string()
                }
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
            get_total_ram,
            set_ram_usage,
            set_config,
            get_username,
            set_username,
            get_ram_usage,
            toggle_mod,
            delete_mod,
            save,
            get_mods,
            download_version,
            get_profiles,
            get_installed_versions,
            get_non_installed_versions,
            create_offline_profile,
            set_language,
            get_categorized_versions,
            get_language,
            install_mod_from_local,
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
async fn save(state: State<'_, AppState>) -> Void {
    let cfg = state.config.write().await;
    cfg.write_to_file();
    Ok(())
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
async fn set_ram_usage(state: State<'_, AppState>, ram_usage: u64) -> Void {
    let mut config = state.config.write().await;
    config.launch_options.ram_usage = ram_usage;
    Ok(())
}
#[command]
async fn get_ram_usage(state: State<'_, AppState>) -> Returns<u64> {
    Ok(state.config.read().await.launch_options.ram_usage)
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
async fn set_language(state: State<'_, AppState>, lang: String) -> Void {
    let mut config = state.config.write().await;
    config.launcher_settings.language = lang;
    Ok(())
}
#[command]
async fn get_language(state: State<'_, AppState>) -> Returns<String> {
    let cfg = state.config.read().await;
    Ok(cfg.launcher_settings.language.clone())
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
    println!(
        "DEBUG: Downloading version {} from 9craft mirror",
        version_loader.id
    );
    if version_loader.base == FORGE {
        println!(
            "DEBUG: Forge version detected! {} installing it rn!",
            version_loader.id
        );
        download_forge_version(&version_loader.id, &app_handle, &mir).await?;
    };
    if version_loader.base == FABRIC {
        println!(
            "DEBUG: Fabric version detected! {} installing it rn!",
            version_loader.id
        );
        download_fabric(&version_loader, &mir).await?;
    }

    let version = MinecraftVersion::from_id(version_id);
    let inherited_version = version.get_inherited();
    update_download_status("Downloading version...", &app_handle);
    let cfg = &state.config.read().await;
    downloader::download_version(&version, &app_handle, &*cfg).await?;
    if inherited_version.id != version.id {
        downloader::download_version(&inherited_version, &app_handle, &*cfg).await?;
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
