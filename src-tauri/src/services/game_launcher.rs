use std::fs;
use crate::models::error::{launcher_launch_args_not_found, launcher_version_not_found, Returns, Void};
use crate::models::mirror::mirror_from;
use crate::models::platform::get_current_os;
use crate::models::profiles::get_profile;
use crate::models::versions::MinecraftVersion;
use crate::services::directory_manager::*;
use crate::services::downloader::{Global};
use crate::services::jdk_manager::get_java;
use crate::services::utils;
use crate::services::utils::{extend_once, vec_to_string};
pub use crate::AppState;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::path::{PathBuf, MAIN_SEPARATOR_STR};
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter, Manager};
use log::info;
use crate::models::logger::{error, info};


pub async fn launch_game(app_handle: AppHandle, version: String, global_cache: &Global) -> Void {
    info!("DEBUG: Starting game in {version} ");
    let mut versions = global_cache.versions.iter().filter(|x| x.id == version);
    let ver_res = versions.next();
    let state = &app_handle.state::<AppState>();
    let config = state.config.read().await;
    let tx_err = state.log_tx.clone();
    let tx_out = state.log_tx.clone();
    let mirror = mirror_from(&config.download_settings.mirror);
    match ver_res {
        None => {
            return Err(launcher_version_not_found());
        }
        _ => {}
    }
    let version = ver_res.unwrap();
    let inherited_version = version.get_inherited();
    let inherited_json = inherited_version.load_json();

    let version_id = &version.id;

    let version_id_err_clone = version_id.clone();
    let version_id_out_clone = version_id.clone();


    let inherited_id = &inherited_version.id;
    update_download_status("Reading version metadata...", &app_handle);
    let username = &config.launch_options.username;
    let profile = get_profile(username).unwrap();
    let uid = profile.uuid;
    let version_directory = PathBuf::from(&inherited_version.version_path);
    let json: Value = version.load_json();

    let java_version = inherited_json["javaVersion"]["majorVersion"]
        .as_i64()
        .unwrap_or(8)
        .to_string();
    let java_component = inherited_json["javaVersion"]["component"].as_str().unwrap();
    let game_directory = get_minecraft_directory().display().to_string();
    let asset_directory = get_assets_directory().display().to_string();

    // This is a very old argument that is even removed in the newer versions but still required for launching past versions like 1.0
    let resources_directory = get_minecraft_directory()
        .join("resources")
        .display()
        .to_string();

    let libraries = version.get_libraries();
    let asset_index = inherited_json["assets"].as_str().unwrap().to_string();
    let main_class = json["mainClass"].as_str().unwrap();
    let class_path = version_directory
        .join(format!("{inherited_id}.jar"))
        .to_str()
        .unwrap()
        .to_string();
    let natives = get_natives_folder(&inherited_version.id)
        .to_str()
        .unwrap()
        .to_string();
    update_download(100, "Launching game...", &app_handle);

    let xms = config.launch_options.ram_usage_min.to_string() + "M";
    let xmx = config.launch_options.ram_usage_max.to_string() + "M";
    let java = get_java(java_component.to_string())?;
    let typ = json["type"].as_str().unwrap();
    let run_args_iter = get_launch_args(&json)?;
    let jvm_args = get_jvm_args(&json);
    let run_args_iter_inherited = get_launch_args(&inherited_json)?;
    let run_args_iter_sum = extend_once(run_args_iter, run_args_iter_inherited);
    let run_args = run_args_iter_sum
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
    let mut libraries_str = vec_to_string(libraries, separator.to_string());
    while libraries_str.contains("\\") {
        libraries_str = libraries_str.replace("\\", MAIN_SEPARATOR_STR);
    }
    // println!("{}", libraries_str.to_string());
    let mut child =
    if !jvm_args.is_empty() {
        #[cfg(unix)] /// Fixes permission issue
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&java.get_bin_file()) {
                let mut permissions = metadata.permissions();
                let current_mode = permissions.mode();

                if current_mode & 0o111 == 0 {
                    println!("Adding execute permission to Java binary: {:?}", &java.get_bin_file());
                    permissions.set_mode(current_mode | 0o111);
                    std::fs::set_permissions(&java.get_bin_file(), permissions)
                        .expect("Failed to set execute permissions on Java binary");
                }
            } else {
                panic!("Java binary not found at: {:?}", &java.get_bin_file());
            }
        }
        let mut child_cmd = &mut Command::new(java.get_bin_file());
        for arg in jvm_args.clone() {
            child_cmd  = child_cmd.arg(arg.replace("${natives_directory}", get_natives_folder(&version_id.to_string()).to_str().unwrap())
                .replace("${launcher_name}", &state.launcher_details.name).replace("${launcher_version}", &state.launcher_details.version)
                .replace("${classpath}", format!("{}{}{}", class_path, separator, libraries_str).as_str()))

        }
        // println!("{:?}", child_cmd.get_args());
            child_cmd.arg(main_class)
            .args(&run_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn java process")
    }else {
        #[cfg(unix)] /// Fixes permission issue
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&java.get_bin_file()) {
                let mut permissions = metadata.permissions();
                let current_mode = permissions.mode();

                if current_mode & 0o111 == 0 {
                    println!("Adding execute permission to Java binary: {:?}", &java.get_bin_file());
                    permissions.set_mode(current_mode | 0o111);
                    std::fs::set_permissions(&java.get_bin_file(), permissions)
                        .expect("Failed to set execute permissions on Java binary");
                }
            } else {
                panic!("Java binary not found at: {:?}", &java.get_bin_file());
            }
        }

        Command::new(&java.get_bin_file())
            .arg(format!("-Djava.library.path={}", natives))
            .arg(format!("-Xms{xms}"))
            .arg(format!("-Xmx{xmx}"))
            .arg("-Xms2048M")
            .current_dir(&game_directory)
            .arg("-cp")
            .arg(format!("{}{}{}", class_path, separator, libraries_str))
            .arg(main_class)
            .args(&run_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn java process")
    };

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().unwrap();
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            tx_err.send(error(line, version_id_err_clone.clone())).unwrap();
        }
    });

    // generate_stdout(&mut child);
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                tx_out.send(info(line, version_id_out_clone.clone())).unwrap();
            }
        }
    });

    update_download_status("", &app_handle);
    Ok(())
}
pub fn get_jvm_args(json: &Value) -> Vec<String> {
    let mut vec = Vec::new();
    if let Some(arguments) = json.get("arguments") {
        if let Some(jvm_rules) = arguments.get("jvm").and_then(|v| v.as_array()) {
            for rule in jvm_rules {
                if let Some(arg_str) = rule.as_str() {
                    vec.push(arg_str.to_string());
                }
                else if let Some(obj) = rule.as_object() {
                    if let Some(value) = obj.get("value") {
                        if utils::can_apply_rule(obj) {
                            match value {
                                Value::String(s) => vec.push(s.clone()),
                                Value::Array(arr) => {
                                    for item in arr {
                                        if let Some(s) = item.as_str() {
                                            vec.push(s.to_string());
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    vec
}
pub fn get_launch_args(json: &Value) -> Returns<Vec<String>> {
        if json.get("minecraftArguments").is_none() {
            Ok(json["arguments"]["game"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| v.is_string())
                .map(|v| v.as_str().unwrap().to_string())
                .collect::<Vec<String>>())
        } else if !json.get("minecraftArguments").is_none() {
            Ok(json["minecraftArguments"]
                .as_str()
                .unwrap()
                .split(" ")
                .map(|v| v.to_string())
                .collect::<Vec<String>>())
        } else {
            Err(launcher_launch_args_not_found())
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

    fn verify_game_files(id: &MinecraftVersion) -> bool {
        true
    }
