use crate::config::Config;
use crate::directory_manager::*;
use crate::downloader::{download_forge_version, download_version};
use crate::jdk_manager::get_java;
use crate::profile_manager::get_profile;
use crate::utils::{
    extend_once, get_current_os, is_connected_to_internet,
    vec_to_string,
};
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter};

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub async fn launch_game(app_handle: AppHandle, version: String, config: &Config) {
    let mut versions = config.versions.iter().filter(|x| x.id == version);
    let version = versions.next().unwrap();
    let inherited_version = version.get_inherited();
    let inherited_json = inherited_version.load_json();
    if version.is_forge() && !version.is_installed() {
        println!(
            "DEBUG: Forge version detected! {} installing it rn!",
            version.id
        );
        download_forge_version(&version.id).await;
    }
    if is_connected_to_internet().await {
        update_download_status("Downloading version...", &app_handle);
        download_version(&version, &app_handle).await;
        download_version(&inherited_version, &app_handle).await;
    }
    let version_id = &version.id;
    let inherited_id = &inherited_version.id;
    update_download_status("Reading version metadata...", &app_handle);
    let username = &config.username;
    let profile = get_profile(username).unwrap();
    let uid = profile.uuid;
    let version_directory = PathBuf::from(&inherited_version.version_path);
    let json: Value = version.load_json();

    let java_version = inherited_json["javaVersion"]["majorVersion"]
        .as_i64()
        .unwrap()
        .to_string();

    let game_directory = get_minecraft_directory().display().to_string();
    let asset_directory = get_assets_directory().display().to_string();
    // This is a very old argument that is even removed in the newer versions but still required for launching past versions like 1.0
    let resources_directory = get_minecraft_directory()
        .join("resources")
        .display()
        .to_string();

    let libraries = version.get_libraries();
    let asset_index = inherited_json["assetIndex"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let main_class = json["mainClass"].as_str().unwrap();
    let class_path = version_directory
        .join(format!("{inherited_id}.jar"))
        .to_str()
        .unwrap()
        .to_string();
    let libraries_str = vec_to_string(libraries, ";".to_string());
    let natives = get_natives_folder(&inherited_version.id)
        .to_str()
        .unwrap()
        .to_string();
    update_download(100, "Launching game...", &app_handle);

    let ram_usage = config.ram_usage.to_string() + "M";
    let java = get_java(java_version.to_string())
        .await
        .display()
        .to_string();
    let typ = json["type"].as_str().unwrap();
    let mut run_args_iter = get_launch_args(&json);
    let run_args_iter_inherited = get_launch_args(&inherited_json);
    run_args_iter = extend_once(run_args_iter, run_args_iter_inherited);
    let run_args = run_args_iter
        .iter()
        .map(|v| {
            v.replace("${auth_player_name}", username)
                .replace("${version_name}", &version.id)
                .replace("${game_directory}", &game_directory)
                .replace("${assets_root}", &asset_directory)
                .replace("${game_assets}", &resources_directory)
                .replace("${assets_index_name}", &asset_index)
                .replace("${auth_uuid}", &uid.to_string())
                .replace("${auth_access_token}", "accessToken123")
                .replace("${user_properties}", "{}")
                .replace("${user_type}", "legacy")
                .replace("${version_type}", typ)
                .replace("${clientid}", &uuid::Uuid::new_v4().to_string())
                .replace("${auth_xuid}", "0")
        })
        .collect::<Vec<String>>();
    let separator = if get_current_os() == "windows" {
        ";"
    } else {
        ":"
    };
    let mut child = Command::new(&java)
        .arg(format!("-Djava.library.path={}", natives))
        .arg(format!("-Xmx{}", ram_usage))
        .arg("-Xms2048M")
        .current_dir(&game_directory)
        .arg("-cp")
        .arg(format!("{}{}{}", class_path, separator, libraries_str))
        .arg(main_class)
        .args(&run_args)
        // .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn java process");
    let stderr = child.stderr.take().unwrap();
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            println!("[stderr] {}", line);
        }
    });

    let stdout = child.stdout.take().expect("Failed to open stdout");
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("[java stdout] {}", line);
            }
        }
    });

    update_download_status("", &app_handle);
}

pub fn get_launch_args(json: &Value) -> Vec<String> {
    if json.get("minecraftArguments").is_none() {
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
    }
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
