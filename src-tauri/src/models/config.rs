use crate::models::error::{ini_read_err, io_err_read_file, Void};
use crate::models::mirror::Mirror;
use crate::services::directory_manager::get_config_directory;
use serde::{Deserialize, Serialize};
use std::fs;
use crate::models::config::Bool::FALSE;

#[derive(Serialize, Deserialize, Debug)]
pub struct NativeLibraries {
    pub use_custom_glfw: Bool,
    pub glfw_path: Option<String>,
    pub use_custom_openal: Bool,
    pub openal_path: Option<String>,

}
impl Default for NativeLibraries {
    fn default() -> Self {
        Self {
            use_custom_glfw: FALSE,
            glfw_path: None,
            use_custom_openal: FALSE,
            openal_path: None,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchOptions {
    pub username: String,
    pub ram_usage_min: u64,
    pub ram_usage_max: u64,
    pub use_dedicated_gpu: Bool,

}
impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            username: "Player".to_string(),
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
}
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub launch_options: LaunchOptions,
    pub launcher_settings: LauncherSettings,
    pub download_settings: DownloadSettings,
    pub native_libraries: NativeLibraries
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
    pub fn write_to_file(&self) -> Void {
        let text = serde_ini::to_string(self).map_err(|x| ini_read_err(x))?;
        fs::write(get_config_directory(), text).map_err(|x| io_err_read_file(x))
    }
}
