use crate::models::error::{AppError, Void};
use crate::AppState;
use tauri::{command, State};

#[command]
pub async fn get_processes(state: State<'_, AppState>) -> Result<Vec<String>, AppError> {
    let proc_manager = &state.process_manager;
    let processes = proc_manager.active_processes.lock().map_err(|e| AppError::ProcessFetchFailed(e.to_string()))?;
    let process_vec: Vec<String> = processes.keys().clone().map(|x| x.clone()).collect();
    Ok(process_vec)
}

#[command]
pub async fn kill_process(state: State<'_, AppState>, selected_process: String) -> Void {
    let proc_manager = &state.process_manager;
    let mut processes = proc_manager.active_processes.lock().map_err(|e| AppError::ProcessFetchFailed(e.to_string()))?;
    let proc_mutex = processes.remove(&selected_process)
        .ok_or(AppError::ProcessNotFound(selected_process.clone()))?;
    let mut selected_proc = proc_mutex.lock().map_err(|x| AppError::Internal(x.to_string()))?;
    selected_proc.kill().map_err(|x| AppError::Internal(x.to_string()))
}
