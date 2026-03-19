use crate::config::{load_config, Config};
use crate::directory_manager::{
    create_necessary_dirs, get_falcon_launcher_directory, get_mods_folder, get_versions_directory,
};
use crate::downloader::{download_fabric, download_forge_version, GLOBAL_CACHE};
use crate::game_launcher::{launch_game, update_download_status};
use crate::mods::mod_manager;
use crate::mods::mod_manager::{load_mods, set_mod_enabled};
use crate::structs::VersionBase::{FABRIC, FORGE};
use crate::structs::{MinecraftVersion, ModInfo, VersionCategory};
use crate::utils::is_connected_to_internet;
use crate::version_manager::{
    download_version_manifest, get_categorized_versions, initialize_versions,
    reload_installed_versions, VersionLoader,
};
use std::env;
use std::fs::create_dir_all;
use std::string::ToString;
use std::sync::LazyLock;
use tauri::async_runtime::{block_on, spawn, Mutex};
use tauri::ipc::private::ResultFutureKind;
use tauri::{command, AppHandle, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_log::{Target, TargetKind};
use tokio::fs::copy;

mod config;
mod directory_manager;
mod downloader;
mod game_launcher;
mod jdk_manager;
mod profile_manager;
mod structs;
mod utils;

mod mods;
mod version_manager;

static CONFIG: LazyLock<Mutex<Config>> = LazyLock::new(|| Mutex::new(config::default_config()));

#[command]
async fn toggle_mod(mod_info: ModInfo, toggle: bool) {
    set_mod_enabled(mod_info, toggle);
}
#[command]
async fn play_button_handler(app: AppHandle, selected_version: String) {
    launch_game(
        app,
        selected_version,
        &*CONFIG.lock().await,
        &*GLOBAL_CACHE.lock().await,
    )
    .await
    .unwrap();
}
#[command]
async fn load_categorized_versions(
    fabric: bool,
    forge: bool,
    neo_forge: bool,
    lite_loader: bool,
) -> Vec<VersionCategory> {
    get_categorized_versions(fabric, forge, neo_forge, lite_loader).await
}
#[command]
async fn get_versions() -> Vec<String> {
    let global = GLOBAL_CACHE.lock().await;
    global
        .versions
        .iter()
        .map(|x| x.id.to_string())
        .clone()
        .collect()
}
#[command]
async fn get_mods() -> Vec<ModInfo> {
    load_mods()
}

#[command]
async fn reload_versions() {}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {
            let _ = _app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(Target::new(TargetKind::Folder {
                    path: get_falcon_launcher_directory(),
                    file_name: Some("logs".to_string()),
                }))
                .build(),
        )
        .setup(|app| {
            let fl_path = get_falcon_launcher_directory();
            let jdk_path = directory_manager::get_launcher_java_directory();
            let _ = create_dir_all(fl_path);
            let _ = create_dir_all(jdk_path);
            spawn(async {
                create_necessary_dirs().await;
                if is_connected_to_internet().await {
                    download_version_manifest().await;
                }
                load_config(&mut *CONFIG.lock().await).await;
            });
            block_on(async {
                reload_installed_versions().await;
            });
            let window = app.handle().get_window("main").unwrap();

            window.center().expect("Failed to center the window");
            window.set_focus().expect("Failed to set window on focus");
            #[cfg(any(windows, target_os = "linux"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                app.deep_link().register("falconLauncher")?;
                app.deep_link().register_all()?;
            }
            app.deep_link().on_open_url(|event| {
                println!("deep link URLs: {:?}", event.urls());
            });
            return Ok(());
        })
        .invoke_handler(tauri::generate_handler![
            play_button_handler,
            get_versions,
            get_total_ram,
            set_ram_usage,
            set_config,
            get_username,
            reload_versions,
            get_ram_usage,
            toggle_mod,
            delete_mod,
            save,
            get_mods,
            download_version,
            get_profiles,
            get_installed_versions,
            get_non_installed_versions,
            create_offline_profile,
            set_language,
            load_categorized_versions,
            get_language,
            install_mod_from_local
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
#[command]
async fn get_total_ram() -> u64 {
    let ram = sys_info::mem_info().unwrap();
    ram.total
}
#[command]
async fn save() {
    let cfg = CONFIG.lock().await;
    cfg.write_to_file();
}
#[command]
async fn set_config(config: Config) {
    let mut cfg = CONFIG.lock().await;
    cfg.launch_options = config.launch_options;
    cfg.launcher_settings = config.launcher_settings;
    cfg.write_to_file();
}

#[command]
async fn set_ram_usage(ram_usage: u64) {
    let mut config = CONFIG.lock().await;
    config.launch_options.ram_usage = ram_usage;
}
#[command]
async fn get_ram_usage() -> u64 {
    CONFIG.lock().await.launch_options.ram_usage
}

#[command]
async fn get_username() -> String {
    CONFIG.lock().await.launch_options.username.clone()
}

#[command]
async fn get_profiles() -> Vec<String> {
    let profiles = profile_manager::get_profiles();
    profiles.iter().map(|x| x.name.clone()).collect()
}

/*
BUG: the function doesn't invoke on call.
*/
#[command]
async fn create_offline_profile(username: String) {
    profile_manager::create_new_profile(username.clone(), false);
    let mut config = CONFIG.lock().await;
    config.launch_options.username = username;
}
#[command]
async fn get_installed_versions() -> Vec<String> {
    let global = GLOBAL_CACHE.lock().await;
    let versions = global.versions.clone();
    versions
        .iter()
        .filter(|x| x.is_installed())
        .map(|x| x.id.clone())
        .collect()
}
#[command]
async fn get_non_installed_versions() -> Vec<String> {
    let global = GLOBAL_CACHE.lock().await;
    let versions = global.versions.clone();
    versions
        .iter()
        .filter(|x| !x.is_installed())
        .map(|x| x.id.clone())
        .collect()
}

#[command]
async fn set_language(lang: String) {
    let mut config = CONFIG.lock().await;
    config.launcher_settings.language = lang;
}
#[command]
async fn get_language() -> String {
    CONFIG.lock().await.launcher_settings.language.clone()
}

#[command]
async fn install_mod_from_local(app: AppHandle) {
    let paths = app
        .dialog()
        .file()
        .add_filter("Minecraft mods".to_string(), &[&"jar", &"disabled"])
        .blocking_pick_files()
        .unwrap();
    for path in paths {
        let p = path.as_path().unwrap();
        let file_name = p.file_name().unwrap().to_str().unwrap();
        let new_path = get_mods_folder().join(file_name);
        copy(p, new_path).await.unwrap();
    }
}
#[command]
async fn delete_mod(mod_info: ModInfo) {
    mod_manager::delete_mod(&mod_info);
}
#[command]
async fn download_version(app_handle: AppHandle, version_loader: VersionLoader) {
    let version_id = version_loader.get_installed_id();

    if version_loader.base == FORGE {
        println!(
            "DEBUG: Forge version detected! {} installing it rn!",
            version_loader.id
        );
        download_forge_version(&version_loader.id, &app_handle).await;
    };
    if version_loader.base == FABRIC {
        println!(
            "DEBUG: Fabric version detected! {} installing it rn!",
            version_loader.id
        );
        download_fabric(&version_loader).await;
    }

    let version = MinecraftVersion::from_id(version_id);
    let inherited_version = version.get_inherited();
    update_download_status("Downloading version...", &app_handle);
    downloader::download_version(&version, &app_handle).await;
    if inherited_version.id != version.id {
        downloader::download_version(&inherited_version, &app_handle).await;
    }
    update_download_status("", &app_handle);
    app_handle
        .dialog()
        .message("Successfully installed the selected version you can now play it")
        .title("Done!")
        .blocking_show();
    let mut global = GLOBAL_CACHE.lock().await;
    global.versions.push(version);
}
