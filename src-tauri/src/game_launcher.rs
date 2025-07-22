use crate::config::Config;
use crate::directory_manager::*;
use crate::downloader::download_version;
use crate::jdk_manager::get_java;
use crate::structs::library_from_value;
use crate::utils::{
    extend_once, get_current_os, is_connected_to_internet, parse_library_name_to_path,
    vec_to_string,
};
use serde::__private::de::Content::ByteBuf;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter};

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub async fn launch_game(app_handle: AppHandle, version: String, config: &Config) {
    let uid = uuid::Uuid::new_v4();
    let mut versions = config.versions.iter().filter(|x| x.id == version);
    let version = versions.next().unwrap();
    let inherited_version = version.get_inherited();
    let inherited_json = inherited_version.load_json();
    if is_connected_to_internet().await {
        update_download_status("Downloading version...", &app_handle);
        download_version(&version, &app_handle).await;
        download_version(&inherited_version, &app_handle).await;
    }
    let version_id = &version.id;
    let inherited_id = &inherited_version.id;
    update_download_status("Reading version metadata...", &app_handle);
    let username = &config.username;

    let version_directory = PathBuf::from(&inherited_version.version_path);
    let json: Value = version.load_json();

    let java_version = inherited_json["javaVersion"]["majorVersion"]
        .as_i64()
        .unwrap()
        .to_string();

    let game_directory = get_minecraft_directory().display().to_string();
    let asset_directory = get_assets_directory().display().to_string();

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
    let mut libraries = get_library_paths(&inherited_json["libraries"]);
    let libraries_2 = get_library_paths(&json["libraries"]);
    libraries = libraries
        .into_iter()
        .filter(|x| {
            let path = PathBuf::from(x);
            let parent = path.parent().unwrap();
            let artifact = parent
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();

            !libraries_2
                .iter()
                .map(|x| {
                    PathBuf::from(x)
                        .parent()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_lowercase()
                        .to_string()
                })
                .collect::<Vec<String>>()
                .contains(&artifact.to_string())
        })
        .collect::<Vec<String>>();
    libraries = extend_once(libraries, libraries_2);
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
fn get_library_paths(value: &Value) -> Vec<String> {
    let libraries_path = get_libraries_directory();
    let mut libraries = vec![];

    for library in value.as_array().unwrap() {
        if library.get("downloads").is_none() {
            let library_name = library["name"].as_str().unwrap();
            let library_path_str =
                parse_library_name_to_path(library_name.to_string()).replace("/", "\\");
            let library_path = PathBuf::from(&library_path_str);
            if library_path.exists() && !libraries.contains(&library_path_str) {
                libraries.push(library_path_str);
            }
            continue;
        } else if library["downloads"].get("artifact").is_none() {
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
                            libraries.push(path.replace("/", "\\"));
                        }
                    }
                }
            }
            continue;
        }
        let library_info = library_from_value(library);
        let os = get_current_os();
        let path = libraries_path
            .join(&library_info.path.as_str().replace("/", "\\"))
            .to_str()
            .unwrap()
            .to_string()
            .to_string();

        if !libraries.contains(&path) {
            libraries.push(path);
        }
    }

    libraries
}
