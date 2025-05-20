use crate::config::{dump, initialize_configuration_file, load};
use crate::game_launcher::launch_game;
use std::fs::create_dir_all;
use tauri::{command, AppHandle, Manager};
use tauri_plugin_prevent_default::Flags;
use tauri_plugin_prevent_default::KeyboardShortcut;
use tauri_plugin_prevent_default::ModifierKey::{CtrlKey, ShiftKey};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod config;
mod directory_manager;
mod downloader;
mod game_launcher;
mod jdk_manager;
mod structs;
mod utils;
mod version_manager;

#[command]
async fn play_button_handler(app: AppHandle, selected_version: String, username: String) {
    launch_game(app, selected_version, username.as_str()).await;
}
#[command]
async fn get_versions() -> Vec<String> {
    utils::load_versions().await
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let fl_path = directory_manager::get_falcon_launcher_directory().unwrap();
    create_dir_all(fl_path).unwrap();
    initialize_configuration_file();
    let prevent = tauri_plugin_prevent_default::Builder::new()
        .shortcut(KeyboardShortcut::with_modifiers("I", &[CtrlKey, ShiftKey]))
        .shortcut(KeyboardShortcut::with_modifiers("E", &[CtrlKey, ShiftKey]))
        .shortcut(KeyboardShortcut::new("F12"))
        .with_flags(Flags::all().difference(Flags::FIND | Flags::RELOAD))
        .build();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(prevent)
        .invoke_handler(tauri::generate_handler![
            play_button_handler,
            get_versions,
            get_total_ram,
            save_username,
            save_ram_usage,
            get_username,
            get_ram_usage
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
async fn save_username(username: String) {
    let mut config = load();
    config.username = username;
    dump(config);
}
#[command]
async fn save_ram_usage(ram_usage: u64) {
    let mut config = load();
    config.ram_usage = ram_usage;
    dump(config);
}
#[tauri::command]
async fn get_ram_usage() -> u64 {
    let mut config = load();
    config.ram_usage
}
#[tauri::command]
async fn get_username() -> String {
    let mut config = load();
    config.username
}
