use crate::utils;

pub async fn download_java(id: &String) {
    let os = utils::get_current_os();
    let mut url = if os == "linux" {
        format!("https://corretto.aws/downloads/latest/amazon-corretto-{id}-x64-windows-jdk.zip")
    } else if os == "linux" {
        format!("https://corretto.aws/downloads/latest/amazon-corretto-{id}-x64-linux-jdk.tar.gz")
    } else {
        format!("https://corretto.aws/downloads/latest/amazon-corretto-{id}-x64-macos-jdk.pkg")
    };

    let resp = reqwest::get(&url).await.unwrap();
    let file_name = url.split("/").last().unwrap();
    //TODO: download file by url with reqwest
    //TODO: Extract the zip and then delete that for better memory management.
}

pub async fn load_java(id: String) {}
