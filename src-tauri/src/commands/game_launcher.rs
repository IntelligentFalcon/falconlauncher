use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::path::{PathBuf, MAIN_SEPARATOR_STR};
use std::process::{Command, Stdio};
use std::str::FromStr;
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
use crate::services::utils::{extend_once, patch_java_permission_linux, vec_to_string};

#[command]
pub async fn play(app_handle: AppHandle, state: State<'_, AppState>, selected_version: String, repair_mode: bool, profile: &str) -> Void {
    log::info!("Launching minecraft {selected_version} ");

    let global_cache = &*GLOBAL_CACHE.lock().await;
    let tx_err = state.log_tx.clone();
    let tx_out = state.log_tx.clone();
    let uid = Uuid::from_str(profile)
        .map_err(|e| ProfileNotFound(format!("Failed to parse uid {profile}: {e}")))?;

    let config = state.config.read().await;
    let launch_options = &config.launch_options;
    let profile = get_profile(&uid).ok_or_else(|| ProfileNotFound(format!("Selected profile {uid} was not found.")))?;
    let username = profile.username;
    let xms = launch_options.ram_usage_min.to_string() + "M";
    let xmx = launch_options.ram_usage_max.to_string() + "M";

    let mut versions = global_cache.versions.iter().filter(|x| x.id == selected_version);
    let version = versions.next().ok_or(AppError::VersionNotFound)?;
    let version_id = &version.id;
    let version_id_err_clone = version_id.clone();
    let version_id_out_clone = version_id.clone();

    let json: Value = version.load_json();

    let inherited_version = version.get_inherited();
    let inherited_json = inherited_version.load_json();
    let inherited_id = &inherited_version.id;

    log::info!("{}", inherited_json);

    // Throw an error if crucial manifest data is missing instead of falling back
    let java_component = inherited_json
        .pointer("/javaVersion/component")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ManifestParseFailed("Missing '/javaVersion/component' in manifest. The JSON might be corrupted.".to_string()))?;

    if repair_mode {
        download_version(&version, &"".to_string(), &app_handle, &state.log_tx, &config).await?;
        download_version(&inherited_version, &"".to_string(), &app_handle, &state.log_tx, &config).await?;
    }

    let java = services::jdk_manager::get_java(java_component.to_string())?;

    let version_directory = PathBuf::from(&inherited_version.version_path);
    let game_directory = get_minecraft_directory().display().to_string();
    let asset_directory = get_assets_directory().display().to_string();
    let resources_directory = get_minecraft_directory()
        .join("resources")
        .display()
        .to_string();

    let libraries = version.get_libraries();

    let asset_index = inherited_json
        .get("assets")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ManifestParseFailed("Missing 'assets' index in manifest. The JSON might be corrupted.".to_string()))?
        .to_string();

    let main_class = json
        .get("mainClass")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ManifestParseFailed("Missing 'mainClass' in manifest. The JSON might be corrupted.".to_string()))?;

    let class_path = version_directory
        .join(format!("{inherited_id}.jar"))
        .to_string_lossy()
        .into_owned();

    let natives = get_natives_folder(&inherited_version.id)
        .to_string_lossy()
        .into_owned();

    let typ = json
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ManifestParseFailed("Missing 'type' in manifest. The JSON might be corrupted.".to_string()))?;

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

    // Logging is genuinely optional, so keeping this as a safe fallback
    if let Some(argument) = json.pointer("/logging/client/argument").and_then(|v| v.as_str()) {
        if let Some(file_id) = json.pointer("/logging/client/file/id").and_then(|v| v.as_str()) {
            let logger_path = version_directory.join(file_id);
            run_args.push(
                argument.replace("{path}", &logger_path.to_string_lossy())
            );
        }
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

    patch_java_permission_linux(&java);

    let mut child_cmd = Command::new(java.get_bin_file());
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
            child_cmd.arg(
                arg.replace(
                    "${natives_directory}",
                    &get_natives_folder(&version_id.to_string()).to_string_lossy()
                )
                    .replace("${launcher_name}", &state.launcher_details.name)
                    .replace("${launcher_version}", &state.launcher_details.version)
                    .replace(
                        "${classpath}",
                        &format!("{}{}{}", class_path, separator, libraries_str),
                    ),
            );
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

    let _ = tx_out.send(info(
        format!("Loaded libraries: {libraries_str}\n\n"),
        version_id_out_clone.clone(),
    ));
    let _ = tx_out.send(info(
        format!("Game arguments: {run_args_str}"),
        version_id_out_clone.clone(),
    ));
    let _ = tx_out.send(info(
        format!("JVM arguments: {jvm_args_str}"),
        version_id_out_clone.clone(),
    ));

    let envs = child_cmd
        .get_envs()
        .map(|(k, v)| {
            format!(
                "{}={}",
                k.to_string_lossy(),
                v.map(|val| val.to_string_lossy().into_owned()).unwrap_or_default()
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let _ = tx_out.send(info(
        format!("Environments: {envs}"),
        version_id_out_clone.clone(),
    ));

    let mut child = child_cmd
        .spawn()
        .map_err(|e| AppError::Internal(format!("Failed to spawn java process: {}", e)))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Internal("Failed to open stdout".to_string()))?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::Internal("Failed to open stderr".to_string()))?;

    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            let _ = tx_err.send(error(line, version_id_err_clone.clone()));
        }
    });

    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            let _ = tx_out.send(info(line, version_id_out_clone.clone()));
        }
    });

    Ok(())
}