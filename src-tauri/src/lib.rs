use crate::downloader::load_version_manifest;
use crate::game_launcher::launch_game;
use tauri::{command, Manager};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod directory_manager;
mod downloader;
mod game_launcher;
mod structs;
mod utils;
#[command]
fn play_button_handler(selected_version: String, username: String) {
    println!("test");
    launch_game(selected_version, username.as_str());
}

#[command]
fn get_versions() -> Vec<String> {
    utils::load_versions()
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![play_button_handler, get_versions])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
