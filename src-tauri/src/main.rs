// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::downloader::download_forge_version;
use crate::mod_manager::load_mods;
use discord_sdk::activity::{ActivityBuilder, Assets};
use discord_sdk::DiscordHandler;
use std::fs::File;
use std::time::Duration;
use tauri::async_runtime::block_on;
use tauri::ipc::RuntimeCapability;

mod config;
mod directory_manager;
mod downloader;
mod game_launcher;

mod jdk_manager;
mod structs;
mod utils;

mod mod_manager;
mod profile_manager;
mod version_manager;

#[allow(unused_imports)]
fn main() {
    falcon_lib::run()
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

#[test]
fn test_activity() {
    block_on(async {
        let (wheel, handler) = discord_sdk::wheel::Wheel::new(Box::new(|err| {
            eprintln!("Error: {:?}", err);
        }));
        let discord = discord_sdk::Discord::new(
            discord_sdk::DiscordApp::PlainId(1404037939305910465),
            discord_sdk::Subscriptions::ACTIVITY,
            Box::new(handler), // <-- FIX HERE
        )
        .expect("Error creating discord application");
        discord
            .update_activity(
                ActivityBuilder::default()
                    .state("In the Launcher")
                    .details("Browsing Minecraft versions")
                    .assets(Assets::default().large("launchericon", Some("Falcon Launcher"))),
            )
            .await
            .expect("TODO: panic message");
        loop {
            tokio::time::sleep(Duration::from_millis(16)).await;
        }
    });
}
