// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::directory_manager::get_minecraft_directory;
use crate::downloader::download_version;
use crate::game_launcher::launch_game;
use crate::version_manager::download_version_manifest;
use tauri::async_runtime::block_on;

mod config;
mod directory_manager;
mod downloader;
mod game_launcher;
mod structs;
mod utils;

mod version_manager;
fn main() {
    falcon_lib::run()
}

#[test]
fn test_envs() {
    get_minecraft_directory().expect("Minecraft dir was not found");
}

#[test]
fn test_manifest() {
    block_on(async {
        let manifest = download_version_manifest().await;
        println!("{:?}", manifest)
    });
}
