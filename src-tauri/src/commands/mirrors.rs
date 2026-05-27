use crate::models::error::{Returns, Void};
use crate::models::mirror::Mirror;
use crate::models::mirror::{mojang_mirror, ninecraft_mirror};
use tauri::{command, AppHandle};
#[command]
pub async fn get_available_mirrors() -> Returns<Vec<Mirror>> {
    Ok(vec![mojang_mirror(), ninecraft_mirror()])
}

#[command]
pub async fn set_mirror(app_handle: AppHandle, mirror: Mirror) -> Void {
    Ok(())
}

#[command]
pub async fn get_mirror(app_handle: AppHandle) -> Returns<Mirror> {
    Ok(ninecraft_mirror())
}