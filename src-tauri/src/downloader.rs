use crate::directory_manager::{get_assets_directory, get_libraries_directory};
use crate::utils;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
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

fn download_assets(value: Value) {
    let id = value["id"].as_str().unwrap();
    let url = value["url"].as_str().unwrap();
    let total_size = value["totalSize"].as_u64().unwrap();
    let size = value["size"].as_u64().unwrap();
    let mut json: Option<Value> = None;
    block_on(async {
        download_file(
            url.to_string(),
            get_assets_directory()
                .expect("Couldn't get minecraft directory")
                .join("indexes")
                .to_str()
                .unwrap()
                .to_string(),
        )
        .await;
        let content = reqwest::get(url)
            .await
            .expect("Failed to download file.")
            .text()
            .await
            .expect("Failed to read file.");
        json =
            Some(serde_json::from_str(content.as_str()).expect("JSON File isn't well formatted."));
    });
    let url_template = "https://resources.download.minecraft.net/{id}/{hash}";
    match json {
        Some(json) => {
            for asset_object in json["objects"].as_array().unwrap() {
                let hash = asset_object["hash"].as_str().unwrap();
                let id = hash[0..2].to_string().clone();
                let url = url_template
                    .replace("{id}", id.as_str())
                    .replace("{hash}", hash)
                    .clone();
            }
        }
        None => {}
    }
}

pub fn download_version(id: String) {
    let manifest = load_version_manifest();
    //TODO: Some tasks like downloading version libraries & assets and stuff should be done here.
}

async fn download_libraries(value: Value) {
    let libraries_path = get_libraries_directory().expect("Getting library path failed");
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
        let library_downloads = library
            .get("downloads")
            .expect("Parsing library_downloads failed");
        let library_path = library_downloads
            .get("path")
            .expect("Parsing library path failed");
        let library_url = library_downloads
            .get("url")
            .expect("Parsing library_url failed");
        let os = utils::get_current_os();
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
                    let path = libraries_path.join(library_path.as_str().unwrap());
                    download_file(
                        library_url.as_str().unwrap().to_string(),
                        path.to_str().unwrap().to_string(),
                    )
                    .await;
                }
            }
            None => {
                let path = libraries_path.join(library_path.as_str().unwrap());
                download_file(
                    library_url.as_str().unwrap().to_string(),
                    path.to_str().unwrap().to_string(),
                )
                .await;
            }
        }
    }
}

async fn download_file(url: String, dest: String) {
    let mut resp = reqwest::get(url).await.expect("Downloading file failed.");
    let mut out = File::create(dest).expect("Unable to create file.");
    out.write_all(resp.chunk().await.unwrap().unwrap().as_ref())
        .expect("Writing file failed.");
}
