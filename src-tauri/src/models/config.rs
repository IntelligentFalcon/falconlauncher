use crate::models::error::AppError;
use crate::models::mirror::Mirror;
use crate::services::directory_manager::get_config_directory;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;
use crate::models::config::Bool::FALSE;
use crate::services::utils;
use serde_with::with_prefix;

with_prefix!(prefix_java "java_");
with_prefix!(prefix_openal "openal_");
with_prefix!(prefix_glfw "glfw_");
const NATIVE_CUSTOM: &str = "custom";
const NATIVE_VERSION_ASSOCIATED: &str = "version_associated";
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NativeChoice {
    pub mode: String, /// "version_associated" | "custom"
    #[serde(default)]
    pub path: String,
}

impl NativeChoice {
    pub fn is_custom(&self) -> bool {
        self.mode == NATIVE_CUSTOM
    }

    pub fn is_version_associated(&self) -> bool {
        self.mode == NATIVE_VERSION_ASSOCIATED
    }
}
impl Default for NativeChoice {
    fn default() -> Self {
        Self {
            mode: "version_associated".to_string(),
            path: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NativeBinaries {
    #[serde(flatten, with="prefix_glfw")]
    pub glfw: NativeChoice,
    #[serde(flatten, with="prefix_openal")]
    pub openal: NativeChoice,
    #[serde(flatten, with="prefix_java")]
    pub java: NativeChoice,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchOptions {
    pub ram_usage_min: u64,
    pub ram_usage_max: u64,
    pub use_dedicated_gpu: Bool,

}
impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            ram_usage_min: 1024,
            ram_usage_max: 2048,
            use_dedicated_gpu: Bool::TRUE,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LauncherSettings {
    pub language: String,
    pub exit_on_launch: Bool,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            exit_on_launch: Bool::FALSE,
        }
    }
}
mod mirror_serialization {
    use super::*;
    use serde::{de, Deserializer, Serializer};
    use crate::models::mirror::mirror_from;

    pub fn serialize<S>(mirror: &Mirror, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&mirror.name)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Mirror, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = String::deserialize(deserializer)?;
        Ok(mirror_from(&name))
    }
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DownloadSettings {
    #[serde(with = "mirror_serialization")]
    pub mirror: Mirror,
    pub proxy: String,
}


#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub launch_options: LaunchOptions,
    pub launcher_settings: LauncherSettings,
    pub download_settings: DownloadSettings,
    pub native_libraries: NativeBinaries
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub enum Bool {
    TRUE,
    #[default]
    FALSE,
}
impl From<Bool> for bool {
    fn from(value: Bool) -> bool {
        match value {
            Bool::TRUE => true,
            Bool::FALSE => false,
        }
    }
}
impl Bool {
    pub fn new(toggle: bool) -> Bool {
        if toggle {
            Bool::TRUE
        } else {
            Bool::FALSE
        }
    }

    pub fn boolean(&self) -> bool {
        match self {
            Bool::TRUE => true,
            Bool::FALSE => false,
        }
    }
}
impl Config {
    pub fn write_to_file(&self) -> Result<(), AppError> {
        let text = serde_ini::to_string(self).map_err(|x| AppError::IniParseFailed(x.to_string()))?;
        fs::write(get_config_directory(), text).map_err(|x| AppError::FileReadFailed(x.to_string()))
    }
}
