use tauri::{command, State};
use crate::AppState;
use crate::models::config::Bool;
use crate::models::error::{Returns, Void};

#[command]
pub async fn set_maximum_ram_usage(state: State<'_, AppState>, ram_usage: u64) -> Void {
    let mut config = state.config.write().await;
    config.launch_options.ram_usage_max = ram_usage;
    Ok(())
}
#[command]
pub async fn get_maximum_ram_usage(state: State<'_, AppState>) -> Returns<u64> {
    Ok(state.config.read().await.launch_options.ram_usage_max)
}

#[command]
pub async fn set_minimum_ram_usage(state: State<'_, AppState>, ram_usage: u64) -> Void {
    let mut config = state.config.write().await;
    config.launch_options.ram_usage_min = ram_usage;
    Ok(())
}
#[command]
pub async fn get_minimum_ram_usage(state: State<'_, AppState>) -> Returns<u64> {
    Ok(state.config.read().await.launch_options.ram_usage_min)
}

#[command]
pub async fn get_language(state: State<'_, AppState>) -> Returns<String> {
    let cfg = state.config.read().await;
    Ok(cfg.launcher_settings.language.clone())
}
#[command]
pub async fn set_language(state: State<'_, AppState>, lang: String) -> Void {
    let mut config = state.config.write().await;
    config.launcher_settings.language = lang;
    Ok(())
}
#[command]
pub async fn should_exit_on_launch(state: State<'_, AppState>) -> Returns<bool> {
    let cfg = state.config.read().await;
    Ok(cfg.launcher_settings.exit_on_launch.boolean().clone())
}

#[command]
pub async fn set_exit_on_launch(state: State<'_, AppState>, toggle: bool) -> Void {

    let mut config = state.config.write().await;
    config.launcher_settings.exit_on_launch = Bool::new(toggle);
    Ok(())
}


#[command]
pub async fn save(state: State<'_, AppState>) -> Void {
    let cfg = state.config.write().await;
    cfg.write_to_file();
    Ok(())
}