use crate::directory_manager::get_launcher_java_directory;
use crate::utils;
use std::fs;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;
use zip_extract::extract;

pub async fn download_java(id: &String) {
    let os = utils::get_current_os();
    let mut url = if os == "windows" {
        format!("https://corretto.aws/downloads/latest/amazon-corretto-{id}-x64-windows-jdk.zip")
    } else if os == "linux" {
        format!("https://corretto.aws/downloads/latest/amazon-corretto-{id}-x64-linux-jdk.tar.gz")
    } else {
        format!("https://corretto.aws/downloads/latest/amazon-corretto-{id}-x64-macos-jdk.tar.gz")
    };

    let file_name = url.split("/").last().unwrap_or("");
    let zip_file_path = get_launcher_java_directory().join(file_name);
    let mut output_folder = get_launcher_java_directory().join(id);
    if output_folder.join("bin").exists() {
        return;
    }
    let resp = reqwest::get(&url).await.unwrap();
    let mut file = File::create(&zip_file_path).unwrap();
    file.write(resp.bytes().await.unwrap().as_ref()).unwrap();
    let mut zip_file = File::open(&zip_file_path).unwrap();
    extract(&zip_file, &mut output_folder, false).expect("Extraction of java zip file failed!");
    remove_file(&zip_file_path).expect("TODO: deletion of zip file failed");
    let dirs = output_folder.read_dir().unwrap();
    for dir in dirs {
        let unwrapped_dir = dir.unwrap();
        if unwrapped_dir.file_type().unwrap().is_dir() {
            for entry in unwrapped_dir.path().read_dir().unwrap() {
                let path = entry.unwrap().path();
                let new_path = path
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join(path.file_name().unwrap());
                println!(
                    "Extracting java file: from {} to {}",
                    path.display(),
                    new_path.display()
                );
                fs::rename(path.as_path(), new_path.as_path()).unwrap();
            }
            fs::remove_dir_all(unwrapped_dir.path()).unwrap();
        }
    }
}

pub async fn get_java(id: String) -> PathBuf {
    download_java(&id).await;
    let os = utils::get_current_os();
    if os == "windows" {
        get_launcher_java_directory()
            .join(&id)
            .join("bin")
            .join("javaw.exe")
    } else {
        get_launcher_java_directory()
            .join(&id)
            .join("bin")
            .join("java")
    }
}
