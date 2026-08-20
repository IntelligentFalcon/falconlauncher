use crate::models::error::AppError;
use crate::models::logger::{error, info};
use crate::models::platform::get_current_os;
use crate::models::profiles::get_profile;
use crate::services::directory_manager::*;
use crate::services::jdk_manager::{download_java, get_java};
use crate::services::utils;
use crate::services::utils::{extend_once, patch_java_permission_linux, vec_to_string};
pub use crate::AppState;
use crate::Global;
use log::info as log_info;
use serde_json::Value;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::path::{PathBuf, MAIN_SEPARATOR_STR};
use std::process::{Command, Stdio};
use std::str::FromStr;
use tauri::{AppHandle};
use uuid::Uuid;
use crate::models::error::AppError::ProfileNotFound;
use crate::services::game_downloader::download_version;

pub(crate) fn apply_dedicated_gpu_env(cmd: &mut Command) {
    cmd.env("DRI_PRIME", "1");
    cmd.env("__NV_PRIME_RENDER_OFFLOAD", "1");
    cmd.env("__GLX_VENDOR_LIBRARY_NAME", "nvidia");
    cmd.env("__VK_LAYER_NV_optimus", "NVIDIA_only");
}

pub fn get_jvm_args(json: &Value) -> Vec<String> {
    let mut vec = Vec::new();
    if let Some(arguments) = json.get("arguments") {
        if let Some(jvm_rules) = arguments.get("jvm").and_then(|v| v.as_array()) {
            for rule in jvm_rules {
                if let Some(arg_str) = rule.as_str() {
                    vec.push(arg_str.to_string());
                } else if let Some(obj) = rule.as_object() {
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

pub fn get_launch_args(json: &Value) -> Result<Vec<String>, AppError> {
    if let Some(minecraft_args) = json.get("minecraftArguments") {
        let args_str = minecraft_args
            .as_str()
            .ok_or_else(|| AppError::LaunchArgsNotFound("minecraftArguments is not a valid string".to_string()))?;

        Ok(args_str.split(' ').map(|v| v.to_string()).collect())
    } else if let Some(arguments) = json.get("arguments") {
        let game_args = arguments
            .get("game")
            .and_then(|g| g.as_array())
            .ok_or_else(|| AppError::LaunchArgsNotFound("arguments.game is missing or not an array".to_string()))?;

        Ok(game_args
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect())
    } else {
        Err(AppError::LaunchArgsNotFound("Could not find launch arguments in the manifest".to_string()))
    }
}