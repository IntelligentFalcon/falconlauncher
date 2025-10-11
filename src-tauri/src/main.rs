// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::downloader::download_forge_version;
use crate::mods::mod_manager::load_mods;
use discord_sdk::activity::{ActivityBuilder, Assets};
use discord_sdk::DiscordHandler;
use std::fs::File;
use std::time::Duration;
use tauri::async_runtime::block_on;
use tauri::ipc::RuntimeCapability;
use crate::mods::modrinth_helper::{search_for_mod, SearchFacet};

mod config;
mod directory_manager;
mod downloader;
mod game_launcher;

mod jdk_manager;
mod structs;
mod utils;

mod mods;

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

#[test]
fn test_facet_helper(){
    block_on(async {
        let results = search_for_mod("gravestone".to_string(), SearchFacet::new().version("1.21").category("forge").project_type("mod").get_str(),
                                     "relevance".to_string(), 0, 0).await;
        println!("{:?}", results.hits[0]);
    });
}