use std::path::PathBuf;
use std::time::Duration;
use log::info;
use tauri::State;
use tokio_util::sync::CancellationToken;
use crate::AppState;
use crate::models::downloader::PipelineProgressTracker;
use crate::models::error::AppError;
use crate::services::utils::{verify_file_existence_with_sha, verify_file_existence_with_size};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter};
const MAX_RETRIES: u32 = 5;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(25);
const CHUNK_INACTIVITY_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn download_file(
    state: &State<'_, AppState>,
    url: String,
    dest: &PathBuf,
    tracker: Option<&PipelineProgressTracker>,
    cancel_token: Option<&CancellationToken>,
) -> Result<(), AppError> {
    check_cancelled(cancel_token)?;

    let file_name = dest
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());

    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| AppError::DirCreateFailed(e.to_string()))?;
    }
    let part_path = dest.with_extension(format!(
        "{}.part",
        dest.extension().unwrap_or_default().to_string_lossy()
    ));

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(&part_path)
        .await
        .map_err(|e| {
            AppError::FileCreateFailed(format!("Unable to open {:?}: {e}", part_path))
        })?;

    let mut downloaded = file.metadata().await.map(|m| m.len()).unwrap_or(0);
    let mut total_size = 0;
    let mut attempts = 0;

    let cleanup_and_cancel = || async {
        let _ = tokio::fs::remove_file(&part_path).await;
        Err(AppError::DownloadCancelled)
    };

    'retry_loop: while attempts < MAX_RETRIES {
        check_cancelled(cancel_token)?;
        let client = state.client.load_full();

        let mut req = client.get(&url).timeout(CONNECT_TIMEOUT);

        if downloaded > 0 {
            req = req.header(reqwest::header::RANGE, format!("bytes={downloaded}-"));
        }
        let resp = if let Some(token) = cancel_token {
            tokio::select! {
                biased;
                _ = token.cancelled() => return cleanup_and_cancel().await,
                res = req.send() => {
                    match res {
                        Ok(r) => r,
                        Err(_) => {
                            attempts += 1;
                            tokio::time::sleep(Duration::from_millis(500 * 2u64.pow(attempts))).await;
                            continue 'retry_loop;
                        }
                    }
                }
            }
        } else {
            match req.send().await {
                Ok(r) => r,
                Err(_) => {
                    attempts += 1;
                    tokio::time::sleep(Duration::from_millis(500 * 2u64.pow(attempts))).await;
                    continue 'retry_loop;
                }
            }
        };

        let status = resp.status();

        if downloaded > 0 && status == reqwest::StatusCode::OK {
            downloaded = 0;
            file.set_len(0)
                .await
                .map_err(|e| AppError::FileWriteFailed(e.to_string()))?;
            file.seek(tokio::io::SeekFrom::Start(0))
                .await
                .map_err(|e| AppError::FileWriteFailed(e.to_string()))?;
            total_size = resp.content_length().unwrap_or(0);
        } else if status == reqwest::StatusCode::PARTIAL_CONTENT {
            total_size = downloaded + resp.content_length().unwrap_or(0);
        } else if status.is_success() {
            total_size = resp.content_length().unwrap_or(0);
        } else {
            return Err(AppError::DownloadFailed(format!(
                "HTTP request failed with status {status} from {url}"
            )));
        }

        let mut writer = BufWriter::with_capacity(64 * 1024, &mut file);
        let mut stream = resp;

        loop {
            let chunk_res = if let Some(token) = cancel_token {
                tokio::select! {
                    biased;
                    _ = token.cancelled() => return cleanup_and_cancel().await,
                    res = tokio::time::timeout(CHUNK_INACTIVITY_TIMEOUT, stream.chunk()) => res,
                }
            } else {
                tokio::time::timeout(CHUNK_INACTIVITY_TIMEOUT, stream.chunk()).await
            };

            let chunk = match chunk_res {
                Ok(Ok(Some(bytes))) => bytes,
                Ok(Ok(None)) => {
                    writer
                        .flush()
                        .await
                        .map_err(|e| AppError::FileWriteFailed(e.to_string()))?;
                    break 'retry_loop;
                }
                Ok(Err(_)) | Err(_) => {
                    let _ = writer.flush().await;
                    attempts += 1;
                    tokio::time::sleep(Duration::from_millis(500 * 2u64.pow(attempts))).await;
                    continue 'retry_loop;
                }
            };

            writer
                .write_all(&chunk)
                .await
                .map_err(|e| AppError::FileWriteFailed(e.to_string()))?;

            downloaded += chunk.len() as u64;

            if let Some(t) = tracker {
                t.report(&file_name, downloaded, total_size);
            }
        }
    }

    if attempts >= MAX_RETRIES {
        let _ = tokio::fs::remove_file(&part_path).await;
        return Err(AppError::DownloadFailed(format!(
            "Failed after {MAX_RETRIES} attempts due to an unstable network: {url}"
        )));
    }

    tokio::fs::rename(&part_path, dest)
        .await
        .map_err(|e| AppError::FileWriteFailed(e.to_string()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = tokio::fs::metadata(dest).await {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            let _ = tokio::fs::set_permissions(dest, permissions).await;
        }
    }

    info!("Download has been completed successfully!");
    Ok(())
}

pub async fn download_file_if_not_exists(
    state: &State<'_, AppState>,
    path: &PathBuf,
    url: String,
    hash: &str,
    size: u64,
    tracker: Option<&PipelineProgressTracker>,
    cancel_token: Option<&CancellationToken>,
) -> Result<(), AppError> {
    check_cancelled(cancel_token)?;

    if !hash.is_empty() {
        if !verify_file_existence_with_sha(path, hash)? {
            download_file(
                state,
                url,
                path,
                tracker,
                cancel_token,
            )
                .await?;
        }

        return Ok(());
    }

    if !verify_file_existence_with_size(
        &path.to_string_lossy().to_string(),
        size,
    )? {
        download_file(
            state,
            url,
            path,
            tracker,
            cancel_token,
        )
            .await?;
    }

    Ok(())
}


pub fn check_cancelled(token: Option<&CancellationToken>) -> Result<(), AppError> {
    if let Some(token) = token {
        if token.is_cancelled() {
            return Err(AppError::DownloadCancelled);
        }
    }

    Ok(())
}