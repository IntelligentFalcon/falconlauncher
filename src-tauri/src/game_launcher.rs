use crate::config::Config;
use crate::directory_manager::{
    get_assets_directory, get_libraries_directory, get_minecraft_directory, get_natives_folder,
    get_version_directory, get_versions_directory,
};
use crate::downloader::download_version;
use crate::structs::library_from_value;
use crate::utils::{get_current_os, vec_to_string, verify_file_existence};
use serde_json::Value;
use std::fmt::format;
use std::fs::File;
use std::process::Command;
use tauri::{AppHandle, Emitter};

//TODO: Customization
pub async fn launch_game(app_handle: AppHandle, version: String, config: &Config) {
    let uid = uuid::Uuid::new_v4();
    app_handle
        .emit("progress", "Downloading version...")
        .unwrap();
    let username = &config.username;
    download_version(version.clone(), &app_handle).await;

    app_handle
        .emit("progress", "Reading version metadata...")
        .unwrap();
    let version_directory = get_version_directory(&version).unwrap();
    println!("Version: {}", version);
    let version_json_path = version_directory
        .join(format!("{version}.json"))
        .to_str()
        .unwrap()
        .to_string();
    println!("Version directory path: {}", version_json_path);
    let file_json = File::open(version_json_path).unwrap();
    let json: Value = serde_json::from_reader(&file_json).unwrap();
    let game_directory = get_minecraft_directory()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let asset_directory = get_assets_directory()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let asset_index = json
        .get("assetIndex")
        .unwrap()
        .get("id")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    // TODO: From some versions the argument key is changed to something else and launcher must be compatible with all newest versisons as well
    let main_class = json.get("mainClass").unwrap().as_str().unwrap();
    let class_path = version_directory
        .join(format!("{version}.jar"))
        .to_str()
        .unwrap()
        .to_string();
    let libraries = get_library_paths(json.get("libraries").unwrap());
    let libraries_str = vec_to_string(libraries, ";".to_string());
    let natives = get_natives_folder(&version)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    app_handle.emit("progress", "Launching game...").unwrap();
    app_handle.emit("progressBar", 100).unwrap();
    let ram_usage = config.ram_usage.to_string() + "M";
    if json.get("minecraftArguments").is_none() {
        let typ = json.get("type").unwrap().as_str().unwrap();
        let run_args = json
            .get("arguments")
            .unwrap()
            .get("game")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v.is_string())
            .map(|v| {
                v.as_str()
                    .unwrap()
                    .to_string()
                    .replace("${auth_player_name}", username)
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

        Command::new("java")
            .arg(format!("-Djava.library.path={natives}"))
            .arg(format!("-Xmx{ram_usage}"))
            .arg("-Xms1G")
            .arg("-cp")
            .arg(format!("{class_path};{libraries_str}"))
            .arg(main_class)
            .args(run_args)
            .spawn()
            .unwrap();
    } else {
        let run_args = json
            .get("minecraftArguments")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
            .replace("${auth_player_name}", username)
            .replace("${version_name}", &version.to_string())
            .replace("${game_directory}", &game_directory)
            .replace("${assets_root}", &asset_directory)
            .replace("${assets_index_name}", &asset_index)
            .replace("${auth_uuid}", &uid.to_string())
            .replace("${auth_access_token}", "0")
            .replace("${user_properties}", "{}")
            .replace("${user_type}", "legacy");

        let mut run = format!(
            "java -Xms2048M -Xmx{ram_usage} -Djava.library.path={natives} -classpath {class_path};{libraries_str} {main_class} {run_args}"
        );
        Command::new("cmd")
            .args(["/C", run.as_str()])
            .spawn()
            .expect("Couldn't launch the game!");
    }
}

fn get_library_paths(value: &Value) -> Vec<String> {
    let libraries_path = get_libraries_directory().unwrap();
    let mut libraries = vec![];
    for library in value.as_array().unwrap() {
        if library.get("downloads").unwrap().get("artifact").is_none() {
            let classifiers = library.get("downloads").unwrap().get("classifiers");
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
