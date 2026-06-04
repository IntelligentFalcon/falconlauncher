use std::collections::HashMap;
use std::fs;
use std::iter::Map;
use std::time::Duration;
use reqwest::Client;
use serde::de::IntoDeserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;
use crate::models::error::Returns;
use crate::services::directory_manager::get_mirrors_dir;

#[derive(Debug,Deserialize,Serialize)]
pub struct Mirror {
    pub name: String,
    pub description: String,
    pub maps: HashMap<String, String>,
}
impl Mirror {
    pub fn parse_url(&self, url: &String) -> String {

        let mut url = url.to_lowercase();
        url = url.replace("http://", "https://");
        let domain = url
            .strip_prefix("https://")
            .unwrap()
            .split("/")
            .next()
            .unwrap();
        let https_domain = format!("https://{domain}/");
        if  !self.maps.contains_key(https_domain.as_str()) {
            return url.clone();
        }
        println!("{}",url);
        if self.maps.contains_key(https_domain.as_str()) {
            url.replace(https_domain.as_str(), &*self.maps[&https_domain])
        } else {
            url.clone()
        }
    }
    /// FIXME: CHANGE THE TEST METHOD TO PING
    pub async fn is_connected(&self) -> bool {
        let mut t = true;
        for url in self.maps.values() {
            let client = Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap();

            println!("REQUESTING {url}");
            let req = client.head(url).send().await;
            if req.is_err() {
                println!("ERR");
                t = false;
            }
            break;
        }
        t
    }
    pub fn write(&self) {
        let content = serde_json::to_string(&self).unwrap();
        fs::write(get_mirrors_dir().join(format!("{}.json", self.name.to_lowercase())),content).expect("failed to write");
    }
}
pub fn mirror(
    name: String,
    description: String,
    launcher_meta: String,
    piston_meta: String,
    piston_data: String,
    resources: String,
    libraries: String,
) -> Mirror {
    let mut maps = HashMap::new();
    maps.insert(
        "https://launchermeta.mojang.com/".to_string(),
        launcher_meta,
    );

    maps.insert("https://piston-meta.mojang.com/".to_string(), piston_meta);
    maps.insert("https://piston-data.mojang.com/".to_string(), piston_data);
    maps.insert(
        "https://resources.download.minecraft.net/".to_string(),
        resources,
    );
    maps.insert("https://libraries.minecraft.net/".to_string(), libraries);
    Mirror { name, description, maps }
}
pub fn ninecraft_mirror() -> Mirror {
    mirror(
        "9Craft".to_string(),
        "Official 9Craft Mirror ".to_string(),
        "https://launchermeta.9craft.ir/".to_string(),
        "https://piston-meta.9craft.ir/".to_string(),
        "https://piston-data.9craft.ir/".to_string(),
        "https://resources-download.9craft.ir/".to_string(),
        "https://libraries-minecraft.9craft.ir/".to_string(),
    )
}

pub fn mojang_mirror() -> Mirror {
    mirror(
        "Official".to_string(),
        "Official Mirror to download games from".to_string(),
        "https://launchermeta.mojang.com/".to_string(),
        "https://piston-meta.mojang.com/".to_string(),
        "https://piston-meta.mojang.com/".to_string(),
        "https://resources.download.minecraft.net/".to_string(),
        "https://libraries.minecraft.net/".to_string(),
    )
}


// pub fn mirror_from_json(json: Value) -> Returns<Mirror>{
//     let mirror: Mirror = serde_json::from_value;
// }

pub fn mirror_from(name: &String) -> Mirror {
    if name == "9craft" {
        ninecraft_mirror()
    } else {
        mojang_mirror()
    }
}
