use crate::config::{load_config, Config};
use crate::game_launcher::{launch_game, update_download_status};
use std::collections::HashMap;

use crate::directory_manager::get_falcon_launcher_directory;
use crate::downloader::download_forge_version;
use crate::structs::VersionBase::FORGE;
use crate::structs::{MinecraftVersion, VersionCategory};
use crate::utils::is_connected_to_internet;
use crate::version_manager::{
    download_version_manifest, get_categorized_versions, VersionInfo, VersionLoader,
};
use native_dialog::{DialogBuilder, MessageLevel};
use serde::de::value::BoolDeserializer;
use serde_json::Value;
use std::fs::create_dir_all;
use std::io::Write;
use std::ops::Deref;
use std::string::ToString;
use std::sync::LazyLock;
use tauri::async_runtime::{block_on, Mutex};
use tauri::{command, AppHandle, LogicalSize, Manager};
use utils::load_versions;

mod config;
mod directory_manager;
mod downloader;
mod game_launcher;
mod jdk_manager;
mod profile_manager;
mod structs;
mod utils;
mod version_manager;

static CONFIG: LazyLock<Mutex<Config>> = LazyLock::new(|| Mutex::new(config::default_config()));

#[command]
async fn play_button_handler(app: AppHandle, selected_version: String) {
    launch_game(app, selected_version, &*CONFIG.lock().await).await;
}
#[command]
async fn load_categorized_versions(
    fabric: bool,
    forge: bool,
    neo_forge: bool,
    lite_loader: bool,
) -> Vec<VersionCategory> {
    get_categorized_versions(fabric, forge, neo_forge, lite_loader).await
}
#[command]
async fn get_versions() -> Vec<String> {
    CONFIG
        .lock()
        .await
        .versions
        .iter()
        .map(|x| x.id.to_string())
        .clone()
        .collect()
}
#[command]
async fn reload_versions() {}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dialog = DialogBuilder::message()
                            .set_level(MessageLevel::Info)
                            .set_title("من را بخوان")
                            .set_text("این یک خروجی آزمایشی از لانچر هستش لطفا اگه از جایی این رو دریافت کردین برای اپدیت های جدید حتما عضو چنل @IntelligentFalcon
                            در تلگرام بشید چون  که هر چیزی توی این خروجی ممکنه تغییر کنه و یا حذف شده باشه و این که انتظار کار نکردن برخی چیزای داخل لانچر رو داشته باشید.").alert()
                            .show()
                            .unwrap();
            let fl_path = get_falcon_launcher_directory();
            let jdk_path = directory_manager::get_launcher_java_directory();
            create_dir_all(fl_path).unwrap();
            create_dir_all(jdk_path).unwrap();

            block_on(async move {
                load_config(&mut *CONFIG.lock().await).await;
                if is_connected_to_internet().await {
                    download_version_manifest().await;
                }

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
            save,
            download_version,
            get_profiles,
            get_installed_versions,
            get_non_installed_versions,
            create_offline_profile,
            set_language,
            load_categorized_versions,
            get_language
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
#[command]
async fn get_total_ram() -> u64 {
    let ram = sys_info::mem_info().unwrap();
    ram.total
}
#[command]
async fn save() {
    let cfg = CONFIG.lock().await;
    cfg.write_to_file()
}
#[command]
async fn set_username(username: String) {
    let mut config = CONFIG.lock().await;
    config.launch_options.username = username;
}
#[command]
async fn set_ram_usage(ram_usage: u64) {
    let mut config = CONFIG.lock().await;
    config.launch_options.ram_usage = ram_usage;
}
#[command]
async fn get_ram_usage() -> u64 {
    CONFIG.lock().await.launch_options.ram_usage
}
#[command]
async fn get_username() -> String {
    CONFIG.lock().await.launch_options.username.clone()
}

#[command]
async fn get_profiles() -> Vec<String> {
    let profiles = profile_manager::get_profiles();
    profiles.iter().map(|x| x.name.clone()).collect()
}
#[command]
async fn create_offline_profile(username: String) {
    profile_manager::create_new_profile(username, false);
}
#[command]
async fn get_installed_versions() -> Vec<String> {
    let conf = CONFIG.lock().await;
    let versions = conf.versions.clone();
    versions
        .iter()
        .filter(|x| x.is_installed())
        .map(|x| x.id.clone())
        .collect()
}
#[command]
async fn get_non_installed_versions() -> Vec<String> {
    let conf = CONFIG.lock().await;
    let versions = conf.versions.clone();
    versions
        .iter()
        .filter(|x| !x.is_installed())
        .map(|x| x.id.clone())
        .collect()
}

#[command]
async fn set_language(lang: String) {
    let mut config = CONFIG.lock().await;
    config.launcher_settings.language = lang;
}
#[command]
async fn get_language() -> String {
    CONFIG.lock().await.launcher_settings.language.clone()
}

#[command]
async fn download_version(app_handle: AppHandle, version_loader: VersionLoader) {
    let version_id = version_loader.get_installed_id();
    if version_loader.base == FORGE {
        println!(
            "DEBUG: Forge version detected! {} installing it rn!",
            version_loader.id
        );
        download_forge_version(&version_loader.id, &app_handle).await;
    };
    let version = MinecraftVersion::from_id(version_id);
    let inherited_version = version.get_inherited();
    update_download_status("Downloading version...", &app_handle);
    downloader::download_version(&version, &app_handle).await;
    downloader::download_version(&inherited_version, &app_handle).await;
    let dialog = DialogBuilder::message()
        .set_title("Done!")
        .set_text("Successfully installed the selected version you can now play it")
        .alert()
        .show()
        .unwrap();
    let mut conf = CONFIG.lock().await;
    conf.versions.push(version);
}
