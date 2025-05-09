use crate::downloader::load_version_manifest;
use tauri::{command, Manager};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod directory_manager;
mod downloader;
mod game_launcher;
mod structs;
mod utils;

#[command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[command]
fn get_versions() -> Vec<String> {
    utils::load_versions()
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_versions])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
