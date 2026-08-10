use chrono::Local;
use serde::Serialize;
use std::collections::VecDeque;
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use log::info;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

#[derive(Clone, Serialize)]
pub struct LogLine {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub channel: String,
}
impl Display for LogLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = format!(
            "[{}] ({}) [{}]: {}",
            self.timestamp, self.channel, self.level, self.message
        );
        write!(f, "{}", str)
    }
}

pub struct LogSenderState(pub mpsc::UnboundedSender<LogLine>);

pub fn init_log_bridge(
    app_handle: AppHandle,
    history: Arc<Mutex<VecDeque<LogLine>>>,
) -> mpsc::UnboundedSender<LogLine> {
    let (tx, mut rx) = mpsc::unbounded_channel::<LogLine>();

    tauri::async_runtime::spawn(async move {
        while let Some(log_line) = rx.recv().await {
            if let Ok(mut guard) = history.lock() {
                guard.push_back(log_line.clone());
                if guard.len() > 1000 {
                    guard.pop_front();
                }
            }
            let _ = app_handle.emit("launcher-log-stream", log_line);
        }
    });

    tx
}
pub fn log(message: String, level: String, channel: String) -> LogLine {
    let ch = channel.clone();
    let log =
    LogLine {
        message,
        level,
        timestamp: Local::now().format("%H:%M:%S").to_string(),
        channel,
    };
    if ch.to_lowercase() == "launcher" {
        info!("{log}");
    }
    log
}

pub fn error(message: String, channel: String) -> LogLine {
    log(message, "error".to_string(), channel)
}

pub fn warning(message: String, channel: String) -> LogLine {
    log(message, "warning".to_string(), channel)
}

pub fn info(message: String, channel: String) -> LogLine {
    log(message, "info".to_string(), channel)
}
