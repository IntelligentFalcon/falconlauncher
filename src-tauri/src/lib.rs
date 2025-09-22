use crate::config::{load_config, Config};
use crate::directory_manager::{
    create_necessary_dirs, get_falcon_launcher_directory, get_mods_folder,
};
use crate::downloader::{download_fabric, download_forge_version};
use crate::game_launcher::{launch_game, update_download_status};
use crate::mod_manager::{load_mods, set_mod_enabled};
use crate::structs::VersionBase::{FABRIC, FORGE};
use crate::structs::{MinecraftVersion, ModInfo, VersionCategory};
use crate::utils::is_connected_to_internet;
use crate::version_manager::{download_version_manifest, get_categorized_versions, VersionLoader};
use std::fs::{create_dir_all, rename};
use std::io::Write;
use std::ops::Deref;
use std::string::ToString;
use std::sync::LazyLock;
use tauri::async_runtime::{block_on, Mutex};
use tauri::{command, AppHandle, LogicalSize, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_log::{Target, TargetKind};
use tokio::fs::copy;

mod config;
mod directory_manager;
mod downloader;
mod game_launcher;
mod jdk_manager;
mod mod_manager;
mod profile_manager;
mod structs;
mod utils;

mod version_manager;

static CONFIG: LazyLock<Mutex<Config>> = LazyLock::new(|| Mutex::new(config::default_config()));

#[command]
async fn toggle_mod(mod_info: ModInfo, toggle: bool) -> Result<(), String> {
    set_mod_enabled(mod_info, toggle);
    Ok(())
}
#[command]
async fn play_button_handler(app: AppHandle, selected_version: String) -> Result<(), String> {
    launch_game(app, selected_version, &*CONFIG.lock().await).await;
    Ok(())
}
#[command]
async fn load_categorized_versions(
    fabric: bool,
    forge: bool,
    neo_forge: bool,
    lite_loader: bool,
)  -> Result<Vec<VersionCategory>, String>  {
    Ok(get_categorized_versions(fabric, forge, neo_forge, lite_loader).await)
}
#[command]
async fn get_versions() -> Result<Vec<String>, String>  {
    Ok(CONFIG
        .lock()
        .await
        .versions
        .iter()
        .map(|x| x.id.to_string())
        .clone()
        .collect())
}
#[command]
async fn get_mods()  -> Result<Vec<ModInfo>, String> {
    Ok(load_mods())
}

#[command]
async fn reload_versions() {}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().build())
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
        .setup(|app| {
            let fl_path = get_falcon_launcher_directory();
            let jdk_path = directory_manager::get_launcher_java_directory();
            let _ = create_dir_all(fl_path);
            let _ = create_dir_all(jdk_path);

            block_on(async move {
                create_necessary_dirs().await;
                if is_connected_to_internet().await {
                    download_version_manifest().await;
                }
                load_config(&mut *CONFIG.lock().await).await;
            });

            let handle = app.handle();
            let window = handle.get_window("main").unwrap();
            let independant_multiplier = 1.2;
            let monitor = window.primary_monitor().unwrap().unwrap();
            let size = monitor.size();
            let width = size.width;
            let height = size.height;
            let aspect_ratio = width as f64 / height as f64;
            let width = (width as f64 / aspect_ratio) * independant_multiplier;
            let height = (height as f64 / aspect_ratio) * independant_multiplier;

            window
                .set_size(LogicalSize::new(width, height))
                .expect("Failed to change the window size");
            window.center().expect("Failed to center the window");
            window
                .set_resizable(false)
                .expect("Failed to rmeove resiazability");
            window
                .set_maximizable(false)
                .expect("Failed to remove maximizablity");
            window.set_focus().expect("Failed to set window on focus");

            return Ok(());
        })
        .invoke_handler(tauri::generate_handler![
            play_button_handler,
            get_versions,
            get_total_ram,
            set_username,
            set_ram_usage,
            get_username,
            reload_versions,
            get_ram_usage,
            toggle_mod,
            save,
            get_mods,
            download_version,
            get_profiles,
            get_installed_versions,
            get_non_installed_versions,
            create_offline_profile,
            set_language,
            load_categorized_versions,
            get_language,
            install_mod_from_local
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
#[command]
async fn get_total_ram()  -> Result<u64, String> {
    let ram = sys_info::mem_info().unwrap();
    Ok(ram.total)
}
#[command]
async fn save()  -> Result<(), String>{
    let cfg = CONFIG.lock().await;
    cfg.write_to_file();
    Ok(())
}
#[command]
async fn set_username(username: String) -> Result<(), String> {
    let mut config = CONFIG.lock().await;
    config.launch_options.username = username;
    Ok(())
}
#[command]
async fn set_ram_usage(ram_usage: u64) -> Result<(), String> {
    let mut config = CONFIG.lock().await;
    config.launch_options.ram_usage = ram_usage;
    Ok(())
}
#[command]
async fn get_ram_usage() -> Result<u64, String> {
    Ok(CONFIG.lock().await.launch_options.ram_usage)
}
#[command]
async fn get_username() -> Result<String, String> {
    Ok(CONFIG.lock().await.launch_options.username.clone())
}

#[command]
async fn get_profiles() -> Result<Vec<String>, String> {
    let profiles = profile_manager::get_profiles();
    Ok(profiles.iter().map(|x| x.name.clone()).collect())
}
#[command]
async fn create_offline_profile(username: String) -> Result<(), String> {
    profile_manager::create_new_profile(username.clone(), false);
    let mut config = CONFIG.lock().await;
    config.launch_options.username = username;
    Ok(())
}
#[command]
async fn get_installed_versions() -> Result<Vec<String>, String> {
    let conf = CONFIG.lock().await;
    let versions = conf.versions.clone();
    Ok(versions
        .iter()
        .filter(|x| x.is_installed())
        .map(|x| x.id.clone())
        .collect())
}
#[command]
async fn get_non_installed_versions() -> Result<Vec<String>, String> {
    let conf = CONFIG.lock().await;
    let versions = conf.versions.clone();
    Ok(versions
        .iter()
        .filter(|x| !x.is_installed())
        .map(|x| x.id.clone())
        .collect())
}

#[command]
async fn set_language(lang: String) -> Result<(), String> {
    let mut config = CONFIG.lock().await;
    config.launcher_settings.language = lang;
    Ok(())
}
#[command]
async fn get_language() -> Result<String, String> {
    Ok(CONFIG.lock().await.launcher_settings.language.clone())
}

#[command]
async fn install_mod_from_local(app: AppHandle) -> Result<(), String> {
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
async fn download_version(
    app_handle: AppHandle,
    version_loader: VersionLoader,
) -> Result<(), String> {
    let version_id = version_loader.get_installed_id();
    if version_loader.base == FORGE {
        println!(
            "DEBUG: Forge version detected! {} installing it rn!",
            version_loader.id
        );
        download_forge_version(&version_loader.id, &app_handle).await;
    };
    if version_loader.base == FABRIC {
        println!(
            "DEBUG: Fabric version detected! {} installing it rn!",
            version_loader.id
        );
        download_fabric(&version_loader).await;
    }
    let version = MinecraftVersion::from_id(version_id);
    let inherited_version = version.get_inherited();
    update_download_status("Downloading version...", &app_handle);
    downloader::download_version(&version, &app_handle).await;
    downloader::download_version(&inherited_version, &app_handle).await;
    let dialog = app_handle
        .dialog()
        .message("Successfully installed the selected version you can now play it")
        .title("Done!")
        .blocking_show();
    let mut conf = CONFIG.lock().await;
    conf.versions.push(version);
    Ok(())
}
