use crate::downloader::load_version_manifest;
use crate::game_launcher::launch_game;
use std::fs::{create_dir_all, exists};
use tauri::{command, AppHandle, Manager};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod config;
mod directory_manager;
mod downloader;
mod game_launcher;
mod structs;
mod utils;

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

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![play_button_handler, get_versions])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
