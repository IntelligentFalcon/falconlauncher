use crate::config::{dump, load_config, Config};
use crate::game_launcher::launch_game;
use std::fs::create_dir_all;
use std::ops::Deref;
use std::string::ToString;
use std::sync::LazyLock;
use tauri::async_runtime::{block_on, Mutex};
use tauri::{command, AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_prevent_default::Flags;
use tauri_plugin_prevent_default::KeyboardShortcut;
use tauri_plugin_prevent_default::ModifierKey::{CtrlKey, ShiftKey};

mod config;
mod directory_manager;
mod downloader;
mod game_launcher;
mod jdk_manager;
mod structs;
mod utils;
mod version_manager;

static CONFIG: LazyLock<Mutex<Config>> = LazyLock::new(|| {
    Mutex::new(Config {
        username: "Steve".to_string(),
        ram_usage: 1024,
        java_path: "java".to_string(),
        versions: Vec::new(),
    })
});

#[command]
async fn play_button_handler(app: AppHandle, selected_version: String) {
    launch_game(app, selected_version, &*CONFIG.lock().await).await;
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
    let prevent = tauri_plugin_prevent_default::Builder::new()
        .shortcut(KeyboardShortcut::with_modifiers("I", &[CtrlKey, ShiftKey]))
        .shortcut(KeyboardShortcut::with_modifiers("E", &[CtrlKey, ShiftKey]))
        .shortcut(KeyboardShortcut::new("F12"))
        .with_flags(Flags::all().difference(Flags::FIND | Flags::RELOAD))
        .build();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(prevent)
        .invoke_handler(tauri::generate_handler![
            play_button_handler,
            get_versions,
            get_total_ram,
            set_username,
            set_ram_usage,
            get_username,
            get_ram_usage,
            save
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
