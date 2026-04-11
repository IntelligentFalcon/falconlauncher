use crate::models::downloader::Manifest;
use crate::services::directory_manager::{get_versions_directory, version_manifest_directory};
use crate::services::downloader::{download_file, GLOBAL_CACHE};
use crate::models::mirror::Mirror;
use crate::models::error::Void;
use crate::models::versions::MinecraftVersion;

pub async fn load_version_manifest(mirror: &Mirror) -> Option<Manifest> {
    // let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    download_version_manifest(mirror).await;
    load_version_manifest_local()
}

pub fn load_version_manifest_local() -> Option<Manifest> {
    let path = version_manifest_directory();
    let text = std::fs::read_to_string(&path);
    serde_json::from_str(text.unwrap().as_str()).expect("Failed to parse json")
}
pub async fn reload_installed_versions() {
    let versions_dir = get_versions_directory().read_dir().unwrap();
    let versions = versions_dir.filter_map(|x| {
        let d = x.unwrap();
        if d.file_type().unwrap().is_file() {
            return None;
        }

        if d.path().read_dir().unwrap().find(|x| {
            let ent = x.as_ref().unwrap();
            ent.file_name().to_str().unwrap().to_lowercase().contains(".json")
        }).is_some() {
            return  Some(MinecraftVersion::from_folder(d.path()));
        }
        return None;
    }).collect::<Vec<MinecraftVersion>>();
    let mut global = GLOBAL_CACHE.lock().await;
    global.versions = versions;

}

pub async fn initialize_versions(){
    let mut global = GLOBAL_CACHE.lock().await;
    let manifest = load_version_manifest_local().expect("Failed to parse the manifest version");
    for v in &manifest.versions {
        global.versions.push(MinecraftVersion::from_id(v.id.clone()));
    }
}
pub async fn download_version_manifest(mirror: &Mirror) -> Void {
    let url = mirror.parse_url(&"https://launchermeta.mojang.com/mc/game/version_manifest.json".to_string());
    download_file(
        url.to_string(),
        version_manifest_directory().to_str()
            .unwrap()
            .to_string(),
    )
    .await
}

