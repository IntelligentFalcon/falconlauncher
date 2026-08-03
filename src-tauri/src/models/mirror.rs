use crate::models::error::AppError;
use crate::services::directory_manager::get_mirrors_dir;
use reqwest::Client;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use log::info;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Mirror {
    pub name: String,
    pub description: String,
    pub maps: HashMap<String, String>,
}
impl Default for Mirror {
    fn default() -> Self {
        mojang_mirror()
    }
}
impl Mirror {
    pub fn parse_url(&self, url: &String) -> String {
        let mut url = url.clone();
        if url.to_lowercase().starts_with("http://") {
            url.insert("http".len(), 's');
        }

        let https_less_url = if url.to_lowercase().starts_with("https://") {
            &url["https://".len()..].trim().to_string()
        } else {
            &url
        };

        let domain = https_less_url.split("/").next().unwrap().to_lowercase();

        let https_domain = format!("https://{domain}/");

        if !self.maps.contains_key(https_domain.as_str()) {
            return url.clone();
        }
        if self.maps.contains_key(https_domain.as_str()) {
            url.replace(https_domain.as_str(), &*self.maps[&https_domain])
        } else {
            url.clone()
        }
    }
    pub async fn is_connected(&self) -> bool {
        let mut t = true;
        for url in self.maps.values() {
            let client = Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap();
            let req = client.head(url).send().await;
            if req.is_err() {
                info!("ERR");
                t = false;
                break;
            }
        }
        t
    }
    pub fn write(&self) -> Result<(), AppError> {
        let content = serde_json::to_string(&self).unwrap();
        fs::write(
            get_mirrors_dir().join(format!("{}.json", self.name.to_lowercase())),
            content,
        )
        .map_err(|x| AppError::NotImplemented("file write filed".to_string()))
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
    let maps = HashMap::from([
        ("https://launchermeta.mojang.com/".to_string(), launcher_meta),
        ("https://piston-meta.mojang.com/".to_string(), piston_meta),
        ("https://piston-data.mojang.com/".to_string(), piston_data),
        ("https://resources.download.minecraft.net/".to_string(), resources),
        ("https://libraries.minecraft.net/".to_string(), libraries),
    ]);
    Mirror {
        name,
        description,
        maps,
    }
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
        "https://piston-data.mojang.com/".to_string(),
        "https://resources.download.minecraft.net/".to_string(),
        "https://libraries.minecraft.net/".to_string(),
    )
}

pub fn list_mirrors() -> Result<Vec<Mirror>, AppError> {
    let mirrors_dir = get_mirrors_dir();
    let mut vec = Vec::new();
    for entry in mirrors_dir.read_dir().unwrap() {
        if let Ok(entry) = entry {
            if let Ok(content) = fs::read_to_string(get_mirrors_dir().join(entry.file_name())) {
                if let Ok(value) = serde_json::from_str::<Mirror>(content.as_str()) {
                    vec.push(value);
                }
            }
        }
    }
    if vec.iter().find(|x| x.name == mojang_mirror().name).is_none() {
        vec.push(mojang_mirror())
    }
    Ok(vec)

}
pub fn mirror_from(name: &String) -> Mirror {
    let mirrors = list_mirrors();
    match mirrors {
        Ok(val) => {
            if val.iter().filter(|x| x.name == *name).count() == 0 {
                mojang_mirror()
            } else {
                val.iter().find(|x| x.name == *name).unwrap().clone()
            }
        },
        Err(_) => {
            mojang_mirror()
        }
    }
}
