// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::directory_manager::get_minecraft_directory;
use crate::downloader::download_version;
use crate::game_launcher::launch_game;

mod directory_manager;
mod downloader;
mod game_launcher;
mod structs;
mod utils;
fn main() {
    falcon_lib::run()
}


#[test]
fn test_envs() {
    get_minecraft_directory().expect("Minecraft dir was not found");
}
