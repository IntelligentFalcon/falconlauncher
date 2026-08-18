use tauri::{command, State};
use crate::AppState;
use crate::models::config::{Bool, Config};
use crate::models::error::AppError;
use crate::models::profiles::{get_profile, Profile};

#[command]
pub async fn set_maximum_ram_usage(state: State<'_, AppState>, ram_usage: u64) -> Result<(), AppError> {
    let mut config = state.config.write().await;
    config.launch_options.ram_usage_max = ram_usage;
    Ok(())
}
#[command]
pub async fn get_maximum_ram_usage(state: State<'_, AppState>) -> Result<u64, AppError> {
    Ok(state.config.read().await.launch_options.ram_usage_max)
}

#[command]
pub async fn set_minimum_ram_usage(state: State<'_, AppState>, ram_usage: u64) -> Result<(), AppError> {
    let mut config = state.config.write().await;
    config.launch_options.ram_usage_min = ram_usage;
    Ok(())
}
#[command]
pub async fn get_minimum_ram_usage(state: State<'_, AppState>) -> Result<u64, AppError> {
    Ok(state.config.read().await.launch_options.ram_usage_min)
}

#[command]
pub async fn get_language(state: State<'_, AppState>) -> Result<String, AppError> {
    let cfg = state.config.read().await;
    Ok(cfg.launcher_settings.language.clone())
}
#[command]
pub async fn set_language(state: State<'_, AppState>, lang: String) -> Result<(), AppError> {
    let mut config = state.config.write().await;
    config.launcher_settings.language = lang;
    Ok(())
}
#[command]
pub async fn should_exit_on_launch(state: State<'_, AppState>) -> Result<bool, AppError> {
    let cfg = state.config.read().await;
    Ok(cfg.launcher_settings.exit_on_launch.boolean().clone())
}

#[command]
pub async fn set_exit_on_launch(state: State<'_, AppState>, toggle: bool) -> Result<(), AppError> {

    let mut config = state.config.write().await;
    config.launcher_settings.exit_on_launch = Bool::new(toggle);
    Ok(())
}


#[command]
pub async fn save(state: State<'_, AppState>) -> Result<(), AppError> {
    let cfg = state.config.write().await;
    cfg.write_to_file()?;
    Ok(())
}

#[command]
pub async fn get_total_ram() -> Result<u64, AppError> {
    let ram = sys_info::mem_info().unwrap();
    Ok(ram.total / 1000)
}

#[command]
pub async fn set_config(state: State<'_, AppState>, config: Config) -> Result<(), AppError> {
    let mut cfg = state.config.write().await;
    cfg.launch_options = config.launch_options;
    cfg.launcher_settings = config.launcher_settings;
    cfg.write_to_file()?;
    Ok(())
}


