use crate::config::{dump, load_config, Config};
use crate::game_launcher::launch_game;

use std::fs::create_dir_all;
use std::ops::Deref;
use std::string::ToString;
use std::sync::LazyLock;
use tauri::async_runtime::{block_on, Mutex};
use tauri::{command, AppHandle, LogicalSize, Manager};
use tauri_plugin_dialog::DialogExt;
use utils::load_versions;
use version_manager::VersionInfo;

mod config;
mod directory_manager;
mod downloader;
mod game_launcher;
mod jdk_manager;
mod profile_manager;
mod structs;
mod utils;
mod version_manager;

static CONFIG: LazyLock<Mutex<Config>> = LazyLock::new(|| {
    Mutex::new(Config {
        username: "Steve".to_string(),
        ram_usage: 1024,
        java_path: "java".to_string(),
        versions: Vec::new(),
        show_old_versions: true,
        show_snapshots: true,
    })
});

#[command]
async fn play_button_handler(app: AppHandle, selected_version: String) {
    launch_game(app, selected_version, &*CONFIG.lock().await).await;
    //
}

#[command]
async fn reload_versions() {
    let mut config = CONFIG.lock().await;
    config.versions = load_versions(config.show_snapshots, config.show_old_versions).await;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let fl_path = directory_manager::get_falcon_launcher_directory();
    let jdk_path = directory_manager::get_launcher_java_directory();
    create_dir_all(fl_path).unwrap();

    create_dir_all(jdk_path).unwrap();
    block_on(async move {
        load_config(&mut *CONFIG.lock().await).await;
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.dialog().message("این یک خروجی آزمایشی از لانچر هستش لطفا اگه از جایی این رو دریافت کردین برای اپدیت های جدید حتما عضو چنل @IntelligentFalcon
در تلگرام بشید چون  که هر چیزی توی این خروجی ممکنه تغییر کنه و یا حذف شده باشه و این که انتظار کار نکردن برخی چیزای داخل لانچر رو داشته باشید.")
                .title("من را بخوان!")
                .kind(tauri_plugin_dialog::MessageDialogKind::Info)
                .blocking_show();
            let handle = app.handle();
            let window = handle.get_window("main").unwrap();
            let independant_multiplier = 1.2;
            let monitor = window.primary_monitor().unwrap().unwrap();
            let size = monitor.size();
            let aspect_ratio = size.width as f64 / size.height as f64;
            let width = (size.width as f64 / aspect_ratio) * independant_multiplier;
            let height = (size.height as f64 / aspect_ratio) * independant_multiplier;

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
            set_allow_snapshot,
            set_allow_old_versions,
            get_allow_old_versions,
            get_allow_snapshot,
            get_username,
            reload_versions,
            get_ram_usage,
            save,
            get_profiles,
            get_installed_versions,
            get_non_installed_versions,
            create_offline_profile
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
    dump(&cfg);
}
#[command]
async fn set_username(username: String) {
    let mut config = CONFIG.lock().await;
    config.username = username;
}
#[command]
async fn set_ram_usage(ram_usage: u64) {
    let mut config = CONFIG.lock().await;
    config.ram_usage = ram_usage;
}
#[command]
async fn get_ram_usage() -> u64 {
    CONFIG.lock().await.ram_usage
}
#[command]
async fn get_username() -> String {
    CONFIG.lock().await.username.clone()
}
#[command]
async fn set_allow_snapshot(enabled: bool) {
    let mut config = CONFIG.lock().await;
    config.show_snapshots = enabled;
}

#[command]
async fn get_allow_snapshot() -> bool {
    CONFIG.lock().await.show_snapshots
}

#[command]
async fn set_allow_old_versions(enabled: bool) {
    let mut config = CONFIG.lock().await;
    config.show_old_versions = enabled;
}

#[command]
async fn get_allow_old_versions() -> bool {
    CONFIG.lock().await.show_old_versions
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
        .map(|x| x.id.clone()).collect()
}
#[command]
async fn get_non_installed_versions() -> Vec<String> {
    let conf = CONFIG.lock().await;
    let versions = conf.versions.clone();
    versions
        .iter()
        .filter(|x| !x.is_installed())
        .map(|x| x.id.clone()).collect()
}
