use crate::downloader::load_version_manifest;
use crate::game_launcher::launch_game;
use std::fs::{create_dir_all, exists};
use tauri::{command, AppHandle, Manager};
use tauri_plugin_prevent_default::Flags;
use tauri_plugin_prevent_default::KeyboardShortcut;
use tauri_plugin_prevent_default::ModifierKey::{CtrlKey, ShiftKey};

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

    let prevent = tauri_plugin_prevent_default::Builder::new()
  .shortcut(KeyboardShortcut::with_modifiers("I", &[CtrlKey, ShiftKey]))
  .shortcut(KeyboardShortcut::with_modifiers("E", &[CtrlKey, ShiftKey]))
  .shortcut(KeyboardShortcut::new("F12"))
  .with_flags(Flags::all().difference(Flags::FIND | Flags::RELOAD))
  .build();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init()).plugin(prevent)
        .invoke_handler(tauri::generate_handler![play_button_handler, get_versions])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
