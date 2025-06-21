use crate::config::Config;
use crate::directory_manager::*;
use crate::downloader::download_version;
use crate::jdk_manager::get_java;
use crate::structs::library_from_value;
use crate::utils::{get_current_os, is_connected_to_internet, vec_to_string};
use serde_json::Value;
use std::fs::File;
use std::os::windows::process::CommandExt;
use std::process::Command;
use tauri::{AppHandle, Emitter};

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub async fn launch_game(app_handle: AppHandle, version: String, config: &Config) {
    let uid = uuid::Uuid::new_v4();
    if is_connected_to_internet().await {
        update_download_status("Downloading version...", &app_handle);
        download_version(version.clone(), &app_handle).await;
    }
    update_download_status("Reading version metadata...", &app_handle);
    let username = &config.username;

    let version_directory = get_version_directory(&version);
    println!("Version: {}", version);
    let version_json_path = version_directory
        .join(format!("{version}.json"))
        .to_str()
        .unwrap()
        .to_string();
    println!("Version directory path: {}", version_json_path);
    let file_json = File::open(version_json_path).unwrap();
    let json: Value = serde_json::from_reader(&file_json).unwrap();

    let java_version = json["javaVersion"]["majorVersion"]
        .as_i64()
        .unwrap()
        .to_string();

    let game_directory = get_minecraft_directory().display().to_string();
    let asset_directory = get_assets_directory().display().to_string();

    let asset_index = json["assetIndex"]["id"].as_str().unwrap().to_string();
    let main_class = json["mainClass"].as_str().unwrap();
    let class_path = version_directory
        .join(format!("{version}.jar"))
        .to_str()
        .unwrap()
        .to_string();
    let libraries = get_library_paths(&json["libraries"]);
    let libraries_str = vec_to_string(libraries, ";".to_string());
    let natives = get_natives_folder(&version).to_str().unwrap().to_string();
    update_download(100, "Launching game...", &app_handle);

    let ram_usage = config.ram_usage.to_string() + "M";
    let java = get_java(java_version.to_string())
        .await
        .display()
        .to_string();
    let typ = json["type"].as_str().unwrap();
    let run_args_iter = if json.get("minecraftArguments").is_none() {
        json["arguments"]["game"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v.is_string())
            .map(|v| v.as_str().unwrap().to_string())
            .collect::<Vec<String>>()
    } else {
        json["minecraftArguments"]
            .as_str()
            .unwrap()
            .split(" ")
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
    };
    let run_args = run_args_iter
        .iter()
        .map(|v| {
            v.replace("${auth_player_name}", username)
                .replace("${version_name}", &version.to_string())
                .replace("${game_directory}", &game_directory)
                .replace("${assets_root}", &asset_directory)
                .replace("${assets_index_name}", &asset_index)
                .replace("${auth_uuid}", &uid.to_string())
                .replace("${auth_access_token}", "0")
                .replace("${user_properties}", "{}")
                .replace("${user_type}", "legacy")
                .replace("${version_type}", typ)
                .replace("${clientid}", &uuid::Uuid::new_v4().to_string())
                .replace("${auth_xuid}", "0")
        })
        .collect::<Vec<String>>();

    Command::new(format!("{}", java))
        .arg(format!("-Djava.library.path={natives}"))
        .arg(format!("-Xmx{ram_usage}"))
        .arg("-Xms2048M")
        .arg("-cp")
        .arg(format!("{class_path};{libraries_str}"))
        .arg(main_class)
        .args(run_args)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .unwrap();
    update_download_status("", &app_handle);
}
pub fn update_download_bar(progress: i64, app_handle: &AppHandle) {
    app_handle.emit("progressBar", progress).unwrap();
}
pub fn update_download_status(text: &str, app_handle: &AppHandle) {
    app_handle.emit("progress", text).unwrap();
}
pub fn update_download(progress: i64, text: &str, app_handle: &AppHandle) {
    app_handle.emit("progress", text).unwrap();
    app_handle.emit("progressBar", progress).unwrap();
}
fn get_library_paths(value: &Value) -> Vec<String> {
    let libraries_path = get_libraries_directory();
    let mut libraries = vec![];
    for library in value.as_array().unwrap() {
        if library["downloads"].get("artifact").is_none() {
            let classifiers = &library["downloads"].get("classifiers");
            let os = get_current_os();
            match classifiers {
                None => {}
                Some(val) => {
                    let natives = val.get(format!("natives-{os}"));
                    match natives {
                        None => {}
                        Some(_) => {
                            let path = libraries_path
                                .join(natives.unwrap().get("path").unwrap().as_str().unwrap())
                                .to_str()
                                .unwrap()
                                .to_string();
                            libraries.push(path);
                        }
                    }
                }
            }
            continue;
        }
        let library_info = library_from_value(library);
        let os = get_current_os();
        let path = libraries_path
            .join(&library_info.path.as_str())
            .to_str()
            .unwrap()
            .to_string()
            .to_string();
        libraries.push(path);
    }
    libraries
}
