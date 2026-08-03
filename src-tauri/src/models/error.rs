use std::fmt::format;
use serde::{Deserialize, Serialize};
use std::io::Error;
use std::path::PathBuf;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvokeError<T> {
    pub code: String,
    pub data: Option<T>,
}

pub type EmptyError = InvokeError<String>;

/// Result<T, InvokeError<E>>
pub type ReturnsAndErrorType<T, E> = Result<T, InvokeError<E>>;
/// Result<T, InvokeError<()>>
pub type Returns<T> = ReturnsAndErrorType<T, String>;
/// Result<(), InvokeError<E>>
pub type VoidErrorType<E> = ReturnsAndErrorType<(), E>;
/// Result<(), InvokeError<()>>
pub type Void = VoidErrorType<String>;

pub fn io_error_data<T>(code: &str, data: Option<T>) -> InvokeError<T> {
    InvokeError {
        code: code.to_string(),
        data,
    }
}

pub fn io_error(code: &str) -> EmptyError {
    io_error_data(code, None)
}

pub fn io_err_permission(err: std::io::Error) -> InvokeError<String> {
    io_error_data("ERROR_ACCESS_DENIED", Some(err.to_string()))
}

pub fn io_err_create_file(file_name: String, err: std::io::Error) -> InvokeError<String> {
    io_error_data("ERROR_FILE_CREATE_FAILED", Some(err.to_string()))
}

pub fn io_err_rename_file(file_name: String, error: std::io::Error) -> InvokeError<String> {
    io_error_data("ERROR_FILE_RENAME_FAILED", Some(error.to_string()))
}

pub fn io_err_read_file(err: std::io::Error) -> InvokeError<String> {
    io_error_data("ERROR_FILE_READ_FAILED", Some(err.to_string()))
}

pub fn io_err_buffer_read(err: std::io::Error) -> InvokeError<String> {
    io_error_data("ERROR_BUFFER_READ_FAILED", Some(err.to_string()))
}

pub fn json_read_err(err: serde_json::Error) -> InvokeError<String> {
    InvokeError {
        code: "ERROR_JSON_PARSE_FAILED".to_string(),
        data: Some(err.to_string()),
    }
}

pub fn ini_read_err(err: serde_ini::ser::Error) -> InvokeError<String> {
    InvokeError {
        code: "ERROR_INI_PARSE_FAILED".to_string(),
        data: Some(err.to_string()),
    }
}

pub fn launcher_error_data<T>(code: &str, data: Option<T>) -> InvokeError<T> {
    InvokeError {
        code: code.to_string(),
        data,
    }
}

pub fn launcher_error(code: &str) -> EmptyError {
    launcher_error_data(code, None)
}

pub fn launcher_manifest_not_found() -> EmptyError {
    launcher_error("ERROR_MANIFEST_NOT_FOUND")
}

pub fn launcher_file_not_found(file: String) -> EmptyError {
    // format!("Failed to load {file}. make sure you are connected to the internet.")
    launcher_error("ERROR_FILE_NOT_FOUND")
}

pub fn launcher_version_not_found() -> EmptyError {
    // "Couldn't find any selected version. might have to try selecting a version before launching the game".to_string()
    launcher_error("ERROR_VERSION_NOT_FOUND")
}

pub fn launcher_launch_args_not_found() -> EmptyError {
    // "Couldn't find launch arguments".to_string()
    launcher_error("ERROR_LAUNCH_ARGS_NOT_FOUND")
}

pub fn launcher_log_history_not_found() -> EmptyError {
    // "Failed to read log history buffer".to_string()
    launcher_error("ERROR_LOG_HISTORY_NOT_FOUND")
}

pub fn request_error(code: &str) -> EmptyError {
    InvokeError {
        code: code.to_string(),
        data: None,
    }
}

pub fn request_error_data<E>(code: &str, e: E) -> InvokeError<E> {
    InvokeError {
        code: code.to_string(),
        data: Some(e),
    }
}

pub fn request_unknown_err(err: reqwest::Error) -> InvokeError<String> {
    request_error_data("ERROR_NETWORK_REQUEST_FAILED", err.to_string())
}

pub fn download_error() -> EmptyError {
    request_error("ERROR_DOWNLOAD_FAILED")
}

pub fn todo_err(comment: &str) -> EmptyError {
    InvokeError {
        code: "ERROR_NOT_IMPLEMENTED".to_string(),
        data: Some(comment.to_string())
    }
}