use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::path::{PathBuf, MAIN_SEPARATOR_STR};
use std::process::{Command, Stdio};
use std::str::FromStr;
use log::info;
use serde_json::Value;
use tauri::{command, AppHandle, State};
use uuid::Uuid;
use crate::{services, AppState, GLOBAL_CACHE};
use crate::models::error::{AppError, Void};
use crate::models::error::AppError::ProfileNotFound;
use crate::models::logger::{error, info};
use crate::models::platform::get_current_os;
use crate::models::profiles::get_profile;
use crate::services::directory_manager::{get_assets_directory, get_minecraft_directory, get_natives_folder};
use crate::services::game_downloader::download_version;
use crate::services::game_launcher::{get_jvm_args, get_launch_args};
use crate::services::utils;
use crate::services::utils::{extend_once, linux_java_permission_fix, vec_to_string};

#[command]
pub async fn play(app_handle: AppHandle, state: State<'_, AppState>, selected_version: String,repair_mode: bool, profile: &str) -> Void {
    info!("Launching minecraft {selected_version} ");

    let global_cache = &*GLOBAL_CACHE.lock().await;
    let tx_err = state.log_tx.clone();
    let tx_out = state.log_tx.clone();
    let uid = Uuid::from_str(profile)
        .map_err(|e| ProfileNotFound(format!("Failed to parse uid {profile}: {e}")))?;

    let config = state.config.read().await;
    let launch_options = &config.launch_options;
    let profile = get_profile(&uid).ok_or(ProfileNotFound(format!("Selected profile {uid} was not found.")))?;
    let username = profile.username;
    let xms = launch_options.ram_usage_min.to_string() + "M";
    let xmx = launch_options.ram_usage_max.to_string() + "M";

    let mut versions = global_cache.versions.iter().filter(|x| x.id == selected_version);
    let ver_res = versions.next();
    let version = ver_res.ok_or(AppError::VersionNotFound)?;
    let version_id = &version.id;
    let version_id_err_clone = version_id.clone();
    let version_id_out_clone = version_id.clone();

    let json: Value = version.load_json();

    let inherited_version = version.get_inherited();
    let inherited_json = inherited_version.load_json();
    let inherited_id = &inherited_version.id;


    /// TODO: better java handling system is required for future release builds
    info!("{}", inherited_json);
    let java_component = inherited_json["javaVersion"]["component"].as_str().unwrap();
    if repair_mode {
        download_version(&version, &"".to_string(), &app_handle, &state.log_tx, &config).await?;
        download_version(&inherited_version, &"".to_string(), &app_handle, &state.log_tx, &config).await?;
    }
    let java = services::jdk_manager::get_java(java_component.to_string())?;

    let version_directory = PathBuf::from(&inherited_version.version_path);
    let game_directory = get_minecraft_directory().display().to_string();
    let asset_directory = get_assets_directory().display().to_string();
    // This is a very old directory that is even removed in the newer versions but still required for launching older versions like 1.0
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
    let typ = json["type"].as_str().unwrap();
    let run_args_iter = get_launch_args(&json)?;
    let mut jvm_args = get_jvm_args(&json);
    let jvm_args_inherited = get_jvm_args(&inherited_json);
    jvm_args = extend_once(jvm_args_inherited, jvm_args);
    let run_args_iter_inherited = get_launch_args(&inherited_json)?;
    let run_args_iter_sum = extend_once(run_args_iter, run_args_iter_inherited);

    let mut run_args = run_args_iter_sum
        .iter()
        .map(|v| {
            v.replace("${auth_player_name}", username.as_str())
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
                .replace("${clientid}", &Uuid::new_v4().to_string())
                .replace("${auth_xuid}", "0")
        })
        .collect::<Vec<String>>();

    if json.pointer("/logging/client/argument").is_some()
        && json.pointer("/logging/client/file/id").is_some()
    {
        let logger_path =
            version_directory.join(json["logging"]["client"]["file"]["id"].as_str().unwrap());
        run_args.push(
            json["logging"]["client"]["argument"]
                .as_str()
                .unwrap()
                .replace("{path}", logger_path.to_str().unwrap()),
        );
    }
    let separator = if get_current_os() == "windows" {
        ";"
    } else {
        ":"
    };
    let mut libraries_str = vec_to_string(libraries, separator.to_string());
    while libraries_str.contains("\\") {
        libraries_str = libraries_str.replace("\\", MAIN_SEPARATOR_STR);
    }
    linux_java_permission_fix(&java);

    let mut child_cmd = &mut Command::new(java.get_bin_file());
    child_cmd.current_dir(&game_directory)
        .arg(format!("-Xms{xms}"))
        .arg(format!("-Xmx{xmx}"));

    if utils::is_wayland() {
        child_cmd
            .env("XDG_SESSION_TYPE", "x11")
            .env("GDK_BACKEND", "x11")
            .env("__GL_THREADED_OPTIMIZATIONS", "0");
    }
    services::game_launcher::apply_dedicated_gpu_env(&mut child_cmd);
    if !jvm_args.is_empty() {
        for arg in jvm_args.clone() {
            child_cmd = child_cmd.arg(
                arg.replace(
                    "${natives_directory}",
                    get_natives_folder(&version_id.to_string())
                        .to_str()
                        .unwrap(),
                )
                    .replace("${launcher_name}", &state.launcher_details.name)
                    .replace("${launcher_version}", &state.launcher_details.version)
                    .replace(
                        "${classpath}",
                        format!("{}{}{}", class_path, separator, libraries_str).as_str(),
                    ),
            )
        }
    } else {
        child_cmd
            .arg(format!("-Djava.library.path={}", natives))
            .arg("-cp")
            .arg(format!("{}{}{}", class_path, separator, libraries_str));
    };
    child_cmd
        .arg(main_class)
        .args(&run_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let run_args_str = run_args.join(" ");
    let jvm_args_str = jvm_args.join(" ");

    tx_out.send(info(
        format!("Loaded libraries: {libraries_str}\n\n"),
        version_id_out_clone.clone(),
    ));
    tx_out.send(info(
        format!("Game arguments: {run_args_str}"),
        version_id_out_clone.clone(),
    ));
    tx_out.send(info(
        format!("JVM arguments: {jvm_args_str}"),
        version_id_out_clone.clone(),
    ));
    let envs = &child_cmd
        .get_envs()
        .map(|x| {
            format!(
                "{}={}",
                x.0.to_str().unwrap(),
                x.1.unwrap_or(OsStr::new("")).to_str().unwrap()
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    tx_out.send(info(
        format!("Environments: {envs}"),
        version_id_out_clone.clone(),
    ));
    let mut child = child_cmd.spawn().expect("Failed to spawn java process");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().unwrap();
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            tx_err
                .send(error(line, version_id_err_clone.clone()))
                .unwrap();
        }
    });

    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                tx_out
                    .send(info(line, version_id_out_clone.clone()))
                    .unwrap();
            }
        }
    });

    Ok(())
}