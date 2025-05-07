use crate::structs;
use crate::structs::AssetIndex;
use serde_json::Value;
use std::env::vars_os;
use tauri::async_runtime::{block_on, spawn};

/// Returns the version_manifest.json file in a Value structure if the parse process was succeed.
pub fn load_version_manifest() -> Option<Value> {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    let mut json: Option<Value> = None;
    block_on(async {
        let result = reqwest::get(url).await.expect("Failed to download file.");
        let text = result.text().await.expect("Failed to read file.");
        json = Some(serde_json::from_str(text.as_str()).expect("JSON File isn't well formatted."));
    });
    json
}

fn load_version(url: String) {
    // TODO: load version json file.
    let mut json = None;
    block_on(async {
        let result = reqwest::get(url).await.expect("Failed to download file.");
        let text = result.text().await.expect("Failed to read file.");
        json = Some(text);
    });

    spawn(async {});
}

fn load_version_assets(asset_index: AssetIndex) {
    // TODO: download assets from assetindex information.
}

pub fn download_version(id: String) {
    let manifest = load_version_manifest();
    //TODO: Some tasks like downloading version libraries & assets and stuff should be done here.
}

fn download_libraries(value: Value) {
    let libraries = value
        .get("libraries")
        .expect("Parsing libraries of version failed!")
        .as_array()
        .expect("Transforming libraries data to array failed");
    for library in libraries {
        let library_name = library
            .get("name")
            .expect("Parsing library_name failed")
            .as_str()
            .expect("Parsing library_name failed");
        let os = get_current_os();
        let rules = library.get("rules");
        match rules {
            Some(rules) => {
                if rules
                    .get("os")
                    .unwrap()
                    .get("name")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    == os
                {
                    //TODO: download library since it requires the same os that user have
                }
            }
            None => {
                //TODO: download library since it doesnt have rules to disallow any os
            }
        }
    }
}

pub fn get_current_os() -> String {
    structs::parse_os(sys_info::os_type().expect("Unsupported Operating System"))
}
fn download_library(library_value: Value) {}
