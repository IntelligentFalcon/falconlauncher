use log::info;
use tauri::{command, State};
use crate::AppState;
use crate::models::error::{launcher_error, launcher_log_history_not_found, todo_err, Returns, Void};
use crate::models::logger::LogLine;

#[command]
pub async fn get_log_history(state: State<'_, AppState>) -> Returns<Vec<LogLine>> {
    if let Ok(guard) = state.log_history.lock() {
        return Ok(guard.iter().cloned().collect());
    }
    Err(launcher_log_history_not_found())
}

#[command]
pub async fn clear_log_history(state: State<'_, AppState>) -> Void{
    if let Ok(mut guard) = state.log_history.lock() {
        guard.clear();
    }
    Err(launcher_log_history_not_found())
}

#[command]
pub async fn clear_log_history_channel(state: State<'_, AppState>, channel: String) -> Void{
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
    Err(todo_err("Failed to read log history buffer"))
}

/// LINUX Debugger for the js side. use the developer console if you are on Windows build to check logs
#[command]
pub async fn debug(text: String) -> Void {
    info!("{}", text);
    Ok(())
}