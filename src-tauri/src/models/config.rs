use crate::models::error::{ini_read_err, io_err_read_file, Void};
use crate::services::directory_manager::get_config_directory;
use serde::{Deserialize, Serialize};
use std::fs;

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
#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadSettings {
    pub mirror: String,
}
impl Default for DownloadSettings {
    fn default() -> Self {
        Self { mirror: "Official".to_string() }
    }
}
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(rename = "LaunchOptions")]
    pub launch_options: LaunchOptions,
    #[serde(rename = "LauncherSettings")]
    pub launcher_settings: LauncherSettings,
    #[serde(rename = "DownloadSettings")]
    pub download_settings: DownloadSettings
}


#[derive(Debug, Deserialize, Serialize, Default)]
pub enum  Bool {
    TRUE,
    #[default]
    FALSE
}
impl From<Bool> for bool {
    fn from(value: Bool) -> bool {
        match value{
            Bool::TRUE => {true}
            Bool::FALSE => {false}
        }
    }
}
impl Bool{
    pub fn new(toggle: bool) -> Bool{
        if toggle {
            Bool::TRUE
        }else {
            Bool::FALSE
        }
    }

    pub fn boolean(&self) -> bool {
        match self{
            Bool::TRUE => {true}
            Bool::FALSE => {false}
        }
    }
}
impl Config {
    pub fn write_to_file(&self) -> Void {
        let text = serde_ini::to_string(self).map_err(|x| ini_read_err(x))?;
        fs::write(get_config_directory(), text).map_err(|x| io_err_read_file(x))
    }
}