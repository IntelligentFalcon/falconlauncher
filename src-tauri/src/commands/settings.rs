use std::sync::Arc;
use crate::models::config::{Bool, Config, NativeChoice};
use crate::models::error::AppError;
use crate::models::java::Java;
use crate::models::profiles::{get_profile, Profile};
use crate::services::utils::create_reqwest_client;
use crate::AppState;
use tauri::{command, State};
use crate::services::directory_manager::auto_detect_javas;

#[command]
pub async fn set_maximum_ram_usage(
    state: State<'_, AppState>,
    ram_usage: u64,
) -> Result<(), AppError> {
    let mut config = state.config.write().await;
    config.launch_options.ram_usage_max = ram_usage;
    Ok(())
}
#[command]
pub async fn get_maximum_ram_usage(state: State<'_, AppState>) -> Result<u64, AppError> {
    Ok(state.config.read().await.launch_options.ram_usage_max)
}

#[command]
pub async fn set_minimum_ram_usage(
    state: State<'_, AppState>,
    ram_usage: u64,
) -> Result<(), AppError> {
    let mut config = state.config.write().await;
    config.launch_options.ram_usage_min = ram_usage;
    Ok(())
}
#[command]
pub async fn get_minimum_ram_usage(state: State<'_, AppState>) -> Result<u64, AppError> {
    Ok(state.config.read().await.launch_options.ram_usage_min)
}

#[command]
pub async fn set_use_dedicated_gpu(
    state: State<'_, AppState>,
    toggle: bool,
) -> Result<(), AppError> {
    let mut config = state.config.write().await;
    config.launch_options.use_dedicated_gpu = Bool::new(toggle);
    Ok(())
}
#[command]
pub async fn should_use_dedicated_gpu(state: State<'_, AppState>) -> Result<bool, AppError> {
    Ok(state
        .config
        .read()
        .await
        .launch_options
        .use_dedicated_gpu
        .boolean())
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
pub async fn get_proxy(state: State<'_, AppState>) -> Result<String, AppError> {
    let cfg = state.config.read().await;
    Ok(cfg.download_settings.proxy.clone())
}

#[command]
pub async fn set_proxy(state: State<'_, AppState>, proxy: String) -> Result<(), AppError> {
    let mut config = state.config.write().await;
    config.download_settings.proxy = proxy;
    state.client.store(Arc::new(create_reqwest_client(&config)?));
    Ok(())
}

#[command]
pub async fn get_java(state: State<'_, AppState>) -> Result<NativeChoice, AppError> {
    let cfg = state.config.read().await;
    Ok(cfg.native_libraries.java.clone())
}

#[command]
pub async fn set_java(state: State<'_, AppState>, java: NativeChoice) -> Result<(), AppError> {
    let mut config = state.config.write().await;
    config.native_libraries.java = java;
    Ok(())
}

#[command]
pub async fn get_openal(state: State<'_, AppState>) -> Result<NativeChoice, AppError> {
    let cfg = state.config.read().await;
    Ok(cfg.native_libraries.openal.clone())
}

#[command]
pub async fn set_openal(state: State<'_, AppState>, openal: NativeChoice) -> Result<(), AppError> {
    let mut config = state.config.write().await;
    config.native_libraries.openal = openal;
    Ok(())
}

#[command]
pub async fn get_glfw(state: State<'_, AppState>) -> Result<NativeChoice, AppError> {
    let cfg = state.config.read().await;
    Ok(cfg.native_libraries.glfw.clone())
}

#[command]
pub async fn set_glfw(state: State<'_, AppState>, glfw: NativeChoice) -> Result<(), AppError> {
    let mut config = state.config.write().await;
    config.native_libraries.glfw = glfw;
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
    let ram = sys_info::mem_info().expect("Failed to find system's total memory! ");
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

#[command]
pub async fn get_auto_detected_java_versions() -> Result<Vec<Java>, AppError> {
    auto_detect_javas()
}
