use crate::structs::AssetIndex;
use serde_json::Value;
use tauri::async_runtime::{block_on, spawn};

/// Returns the version_manifest.json file in a Value structure if the parse process was succeed.
pub fn load_version_manifest() -> Some(Value) {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    let mut json: Option<Value> = None;
    block_on(async {
        let result = reqwest::get(url).await.expect("Failed to download file.");
        let text = result.text().await.expect("Failed to read file.");
        json = Some(serde_json::from_str(text.as_str()).expect("JSON File isn't well formatted."));
    });
    json
}

pub fn load_version(url: String) {
    // TODO: load version json file.
}

fn load_version_assets(asset_index: AssetIndex) {
    // TODO: download assets from assetindex information.
}

pub fn download_version(id: String) {
    let manifest = load_version_manifest();
    //TODO: Some tasks like downloading version libraries & assets and stuff should be done here.
}
