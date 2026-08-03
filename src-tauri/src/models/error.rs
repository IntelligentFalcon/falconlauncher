use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Manifest Missing")]
    ManifestNotFound,

    #[error("File Not Found: {0}")]
    FileNotFound(String),

    #[error("Version Not Found")]
    VersionNotFound,

    #[error("Launch Arguments Missing")]
    LaunchArgsNotFound,

    #[error("Creation Failed: {0}")]
    FileCreateFailed(String),

    #[error("Rename Failed: {0}")]
    FileRenameFailed(String),

    #[error("Read Error: {0}")]
    FileReadFailed(String),

    #[error("Buffer Error: {0}")]
    BufferReadFailed(String),

    #[error("Invalid JSON: {0}")]
    JsonParseFailed(String),

    #[error("Invalid INI: {0}")]
    IniParseFailed(String),

    #[error("Access Denied: {0}")]
    AccessDenied(String),

    #[error("Log Missing")]
    LogHistoryNotFound,

    #[error("Network Error: {0}")]
    NetworkRequestFailed(String),

    #[error("Download Failed")]
    DownloadFailed,

    #[error("Not Implemented: {0}")]
    NotImplemented(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    SerdeIni(#[from] serde_ini::ser::Error),
}

// Since frontend expects `{ "code": "...", "data": "..." }`:
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 2)?;
        
        let (code, data) = match self {
            AppError::ManifestNotFound => ("ERROR_MANIFEST_NOT_FOUND", None),
            AppError::FileNotFound(_) => ("ERROR_FILE_NOT_FOUND", None),
            AppError::VersionNotFound => ("ERROR_VERSION_NOT_FOUND", None),
            AppError::LaunchArgsNotFound => ("ERROR_LAUNCH_ARGS_NOT_FOUND", None),
            AppError::FileCreateFailed(e) => ("ERROR_FILE_CREATE_FAILED", Some(e.to_string())),
            AppError::FileRenameFailed(e) => ("ERROR_FILE_RENAME_FAILED", Some(e.to_string())),
            AppError::FileReadFailed(e) => ("ERROR_FILE_READ_FAILED", Some(e.to_string())),
            AppError::BufferReadFailed(e) => ("ERROR_BUFFER_READ_FAILED", Some(e.to_string())),
            AppError::JsonParseFailed(e) => ("ERROR_JSON_PARSE_FAILED", Some(e.to_string())),
            AppError::IniParseFailed(e) => ("ERROR_INI_PARSE_FAILED", Some(e.to_string())),
            AppError::AccessDenied(e) => ("ERROR_ACCESS_DENIED", Some(e.to_string())),
            AppError::LogHistoryNotFound => ("ERROR_LOG_HISTORY_NOT_FOUND", None),
            AppError::NetworkRequestFailed(e) => ("ERROR_NETWORK_REQUEST_FAILED", Some(e.to_string())),
            AppError::DownloadFailed => ("ERROR_DOWNLOAD_FAILED", None),
            AppError::NotImplemented(e) => ("ERROR_NOT_IMPLEMENTED", Some(e.to_string())),
            
            AppError::Io(e) => ("ERROR_FILE_READ_FAILED", Some(e.to_string())), // Fallback mapping
            AppError::Anyhow(e) => ("ERROR_INTERNAL", Some(e.to_string())),
            AppError::Reqwest(e) => ("ERROR_NETWORK_REQUEST_FAILED", Some(e.to_string())),
            AppError::SerdeJson(e) => ("ERROR_JSON_PARSE_FAILED", Some(e.to_string())),
            AppError::SerdeIni(e) => ("ERROR_INI_PARSE_FAILED", Some(e.to_string())),
        };

        state.serialize_field("code", code)?;
        state.serialize_field("data", &data)?;
        state.end()
    }
}