// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::directory_manager::get_minecraft_directory;
use crate::jdk_manager::{download_java, get_java};
use crate::mod_manager::load_mods;
use crate::version_manager::download_version_manifest;
use tauri::async_runtime::block_on;

mod config;
mod directory_manager;
mod downloader;
mod game_launcher;
mod jdk_manager;
mod structs;
mod utils;

mod mod_manager;
mod version_manager;
mod account_manager;

#[allow(unused_imports)]
fn main() {
    falcon_lib::run()
}

#[test]
fn test_envs() {
    get_minecraft_directory();
}

#[test]
fn test_manifest() {
    block_on(async {
        let manifest = download_version_manifest().await;
        println!("{:?}", manifest)
    });
}
#[test]
fn test_java_downloader() {
    block_on(async {
        let id = "8".to_string();
        download_java(&id).await
    });
}
#[test]
fn test_get_java() {
    block_on(async {
        let id = "8".to_string();
        println!("{}", get_java(id).await.to_str().unwrap());
    })
}

#[test]
fn test_get_mods() {
    let mods = load_mods();
    println!(
        "Loaded {} mods with first one being called {}",
        mods.len(),
        mods[0].mod_id
    );
}

#[test]
fn test_err_url() {
    block_on(async {
        let sample_failure_url = "https://maven.minecraftforge.net/lzma/lzma/0.0.1/lzma-0.0.1.jar";
        let resp_failure = reqwest::get(sample_failure_url).await;
        assert!(resp_failure.unwrap().status().is_success());
        let sample_success_url =
            "https://repo.spongepowered.org/maven/lzma/lzma/0.0.1/lzma-0.0.1.jar";
        let resp_success = reqwest::get(sample_success_url).await;
        assert!(resp_success.unwrap().status().is_success());
    });
}
