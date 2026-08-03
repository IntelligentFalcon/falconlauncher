use log::info;
use tauri::{command, State};
use crate::AppState;
use crate::models::error::AppError;
use crate::models::logger::LogLine;

#[command]
pub async fn get_log_history(state: State<'_, AppState>) -> Result<Vec<LogLine>, AppError> {
    if let Ok(guard) = state.log_history.lock() {
        return Ok(guard.iter().cloned().collect());
    }
    Err(AppError::LogHistoryNotFound)
}

#[command]
pub async fn clear_log_history(state: State<'_, AppState>) -> Result<(), AppError>{
    if let Ok(mut guard) = state.log_history.lock() {
        guard.clear();
    }
    Err(AppError::LogHistoryNotFound)
}

#[command]
pub async fn clear_log_history_channel(state: State<'_, AppState>, channel: String) -> Result<(), AppError>{
    if let Ok(mut guard) = state.log_history.lock() {
        let l = guard.len();
        let mut i = 0;
        while i != guard.len() {
            if guard[i].channel == channel {
                guard.remove(i);
            } else {
                i  += 1;
            }
        }
    }
    //
    Err(AppError::NotImplemented("Failed to read log history buffer".to_string()))
}

/// LINUX Debugger for the js side. use the developer console if you are on Windows build to check logs
#[command]
pub async fn debug(text: String) -> Result<(), AppError> {
    info!("{}", text);
    Ok(())
}