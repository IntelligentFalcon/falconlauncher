use crate::config::{load_config, Config};
use crate::directory_manager::{
    create_necessary_dirs, get_falcon_launcher_directory, get_mods_folder, get_versions_directory,
};
use crate::downloader::{download_fabric, download_forge_version, GLOBAL_CACHE};
use crate::game_launcher::{launch_game, update_download_status};
use crate::mirror::mojang_mirror;
use crate::mods::mod_manager;
use crate::mods::mod_manager::{load_mods, set_mod_enabled};
use crate::structs::error::{InvokeError, Returns, Void};
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

mod mirror;
mod mods;
mod version_manager;

static CONFIG: LazyLock<Mutex<Config>> = LazyLock::new(|| Mutex::new(config::default_config()));
#[command]
async fn toggle_mod(mod_info: ModInfo, toggle: bool) -> Void {
    set_mod_enabled(mod_info, toggle)
}
#[command]
async fn play(app: AppHandle, selected_version: String) -> Void {
    launch_game(
        app,
        selected_version,
        &*CONFIG.lock().await,
        &*GLOBAL_CACHE.lock().await,
    )
    .await
    .unwrap();
    Ok(())
}
#[command]
async fn load_categorized_versions(
    fabric: bool,
    forge: bool,
    neo_forge: bool,
    lite_loader: bool,
) -> Returns<Vec<VersionCategory>> {
    Ok(get_categorized_versions(fabric, forge, neo_forge, lite_loader).await)
}
#[command]
async fn get_versions() -> Returns<Vec<String>> {
    let global = GLOBAL_CACHE.lock().await;
    Ok(global
        .versions
        .iter()
        .map(|x| x.id.to_string())
        .clone()
        .collect())
}
#[command]
async fn get_mods() -> Returns<Vec<ModInfo>> {
    Ok(load_mods())
}

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
                load_config(&mut *CONFIG.lock().await).await;
                if is_connected_to_internet().await {
                    download_version_manifest(&mojang_mirror()).await;
                }
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
            play,
            get_versions,
            get_total_ram,
            set_ram_usage,
            set_config,
            get_username,
            set_username,
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
            install_mod_from_local,
            debug
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// LINUX Debugger for the js side. use the developer console if you are on windows build to check logs
#[command]
async fn debug(text: String) -> Void {
    println!("{}", text);
    Ok(())
}
#[command]
async fn get_total_ram() -> Returns<u64> {
    let ram = sys_info::mem_info().unwrap();
    Ok(ram.total)
}
#[command]
async fn save() -> Void{
    let cfg = CONFIG.lock().await;
    cfg.write_to_file();
    Ok(())
}
#[command]
async fn set_config(config: Config) -> Void{
    let mut cfg = CONFIG.lock().await;
    cfg.launch_options = config.launch_options;
    cfg.launcher_settings = config.launcher_settings;
    cfg.write_to_file();
    Ok(())
}

#[command]
async fn set_ram_usage(ram_usage: u64) -> Void{
    let mut config = CONFIG.lock().await;
    config.launch_options.ram_usage = ram_usage;
    Ok(())
}
#[command]
async fn get_ram_usage() -> Returns<u64> {
    Ok(CONFIG.lock().await.launch_options.ram_usage)
}

#[command]
async fn get_username() -> Returns<String> {
    Ok(CONFIG.lock().await.launch_options.username.clone())
}

#[command]
async fn set_username(username: String) -> Void {
    let mut cfg = CONFIG.lock().await;
    cfg.launch_options.username = username;
    save().await;
    Ok(())
}

#[command]
async fn get_profiles() -> Returns<Vec<String>> {
    let profiles = profile_manager::get_profiles();
    Ok(profiles.iter().map(|x| x.name.clone()).collect())
}

/*
BUG: the function doesn't invoke on call.
*/
#[command]
async fn create_offline_profile(username: String) -> Void {
    let result = profile_manager::create_new_profile(username.clone(), false);
    let mut config = CONFIG.lock().await;
    config.launch_options.username = username;
    Ok(())
}
#[command]
async fn get_installed_versions() -> Returns<Vec<String>> {
    let global = GLOBAL_CACHE.lock().await;
    let versions = global.versions.clone();
    Ok(versions
        .iter()
        .filter(|x| x.is_installed())
        .map(|x| x.id.clone())
        .collect())
}
#[command]
async fn get_non_installed_versions() -> Returns<Vec<String>> {
    let global = GLOBAL_CACHE.lock().await;
    let versions = global.versions.clone();
    Ok(versions
        .iter()
        .filter(|x| !x.is_installed())
        .map(|x| x.id.clone())
        .collect())
}

#[command]
async fn set_language(lang: String) -> Void{
    let mut config = CONFIG.lock().await;
    config.launcher_settings.language = lang;
    Ok(())
}
#[command]
async fn get_language() -> Returns<String> {
    Ok(CONFIG.lock().await.launcher_settings.language.clone())
}

#[command]
async fn install_mod_from_local(app: AppHandle) -> Void{
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
    Ok(())
}
#[command]
async fn delete_mod(mod_info: ModInfo) -> Void{
    mod_manager::delete_mod(&mod_info);
    Ok(())
}
#[command]
async fn download_version(app_handle: AppHandle, version_loader: VersionLoader) -> Void{
    let version_id = version_loader.get_installed_id();
    println!(
        "DEBUG: Downloading version {} from 9craft mirror",
        version_loader.id
    );
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
    let cfg = &CONFIG.lock().await;
    downloader::download_version(&version, &app_handle, &*cfg).await;
    if inherited_version.id != version.id {
        downloader::download_version(&inherited_version, &app_handle, &*cfg).await;
    }
    update_download_status("", &app_handle);
    app_handle
        .dialog()
        .message("Successfully installed the selected version you can now play it")
        .title("Done!")
        .blocking_show();
    let mut global = GLOBAL_CACHE.lock().await;
    global.versions.push(version);
    Ok(())
}
