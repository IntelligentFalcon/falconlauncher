use serde::{Deserialize, Serialize};
use std::io::Error;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct InvokeError<T> {
    pub code: u32,
    pub message: String,
    pub data: Option<T>,
}

pub type EmptyError = InvokeError<()>;

/// Result<T, InvokeError<E>>
pub type ReturnsAndErrorType<T, E> = Result<T, InvokeError<E>>;
/// Result<T, InvokeError<()>>
pub type Returns<T> = ReturnsAndErrorType<T, ()>;
/// Result<(), InvokeError<E>>
pub type VoidErrorType<E> = ReturnsAndErrorType<(), E>;
/// Result<(), InvokeError<()>>
pub type Void = VoidErrorType<()>;

pub fn io_error_data<T>(message: String, code: u32, data: Option<T>) -> InvokeError<T> {
    InvokeError {
        code,
        message: format!("IO Error: {message}"),
        data,
    }
}
pub fn io_error(message: String, code: u32) -> EmptyError {
    io_error_data(message, code, None)
}

pub fn io_err_create_file(file_name: String, err: Error) -> EmptyError {
    io_error(
        format!("Unable to create file: {file_name} more details: \n {err}"),
        100,
    )
}

pub fn io_err_rename_file(file_name: String, error: Error) -> EmptyError {
    io_error(
        format!("Unable to rename file: {file_name} more info: {error}"),
        101,
    )
}
pub fn io_err_read_file(err: Error) -> EmptyError{
    io_error(format!("Unable to read file: {err}"),102)
}
pub fn json_read_err(err: serde_json::Error) -> EmptyError {
    InvokeError {
        code: 103,
        message: err.to_string(),
        data: None,
    }
}

pub fn launcher_error_data<T>(message: String, code: u32, data: Option<T>) -> InvokeError<T> {
    InvokeError {
        code,
        message: format!("Launcher Error: {message}"),
        data,
    }
}
pub fn launcher_error(message: String, code: u32) -> EmptyError {
    launcher_error_data(message, code, None)
}

pub fn launcher_manifest_not_found() -> EmptyError {
    launcher_error(
        "Failed to load version manifest. make sure you are connected to the internet".to_string(),
        1,
    )
}

pub fn launcher_file_not_found(file: String) -> EmptyError {
    launcher_error(
        format!("Failed to load {file}. make sure you are connected to the internet."),
        2,
    )
}

pub fn launcher_version_not_found() -> EmptyError {
    launcher_error("Couldn't find any selected version. might have to try selecting a version before launching the game".to_string(),
    3)
}
pub fn launcher_launch_args_not_found() -> EmptyError {
    launcher_error("Couldn't find launch arguments".to_string(),
    4)
}
pub fn request_error(message: String, code: u32) -> EmptyError {
    InvokeError {
        code,
        message,
        data: None,
    }
}

pub fn download_error(message: String) -> EmptyError {
    request_error(message, 300)
}
