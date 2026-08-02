use log::debug;
use crate::GLOBAL_CACHE;
use crate::models::downloader::Manifest;
use crate::services::directory_manager::{get_versions_directory, version_manifest_directory};
use crate::services::game_downloader::download_file;
use crate::models::mirror::Mirror;
use crate::models::error::{json_read_err, Returns, Void};
use crate::models::versions::MinecraftVersion;

pub async fn load_version_manifest(mirror: &Mirror) -> Returns<Manifest> {
    download_version_manifest(mirror).await;
    load_version_manifest_local()
}

pub fn load_version_manifest_local() -> Returns<Manifest> {
    let path = version_manifest_directory();
    let text = std::fs::read_to_string(&path);
    serde_json::from_str(text.unwrap().as_str()).map_err(|x| json_read_err(x))
}
pub async fn reload_installed_versions() {
    debug!("Beginning of reload_installed_versions");
    let versions_dir = get_versions_directory().read_dir().unwrap();
    debug!("Passed versions_dir successfully");
    let versions = versions_dir.filter_map(|x| {
        let d = x.unwrap();
        if d.file_type().unwrap().is_file() {
            return None;
        }

        if d.path().read_dir().unwrap().find(|x| {
            let ent = x.as_ref().unwrap();
            debug!("{}", ent.path().display());

            ent.file_name().to_str().unwrap().to_lowercase().contains(".json")
        }).is_some() {
            return  Some(MinecraftVersion::from_folder(d.path()));
        }
        return None;
    }).filter(|x| x.is_ok()).map(|x| x.unwrap()).collect::<Vec<MinecraftVersion>>();
    debug!("Passed versions successfully");

    let mut global = GLOBAL_CACHE.lock().await;
    debug!("Locked global_cache variable successfully.");

    global.versions = versions;

}

pub async fn initialize_versions() -> Void{
    let mut global = GLOBAL_CACHE.lock().await;
    let manifest = load_version_manifest_local()?;
    for v in &manifest.versions {
        global.versions.push(MinecraftVersion::from_id(v.id.clone()));
    }
    Ok(())
}
pub async fn download_version_manifest(mirror: &Mirror) -> Void {
    let url = mirror.parse_url(&"https://launchermeta.mojang.com/mc/game/version_manifest.json".to_string());
    download_file(
        url.to_string(),
        &version_manifest_directory().to_str()
            .unwrap()
            .to_string(),
    )
    .await
}

