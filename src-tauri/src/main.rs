// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use tauri::async_runtime::block_on;
use tracing::info;
use falcon_lib::models::mirror::{mojang_mirror, ninecraft_mirror};
use falcon_lib::services::directory_manager::auto_detect_javas;

#[allow(unused_imports)]
fn main() {
    falcon_lib::run()
}
