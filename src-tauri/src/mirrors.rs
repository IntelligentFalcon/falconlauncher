use std::collections::HashMap;
use std::iter::Map;

pub struct Mirror {
    pub maps: HashMap<String, String>,
}
impl Mirror {
    pub fn parse_url(&self, url: &String) -> String{
        if self.maps.contains_key(url) {
            self.maps[url].clone()
        }else {
            url.clone()
        }
    }
}
pub fn mirror(
    launcher_meta: String,
    piston_meta: String,
    piston_data: String,
    resources: String,
    libraries: String,
) -> Mirror {
    let mut maps = HashMap::new();
    maps.insert("https://launchermeta.mojang.com".to_string(), launcher_meta);
    maps.insert("https://piston-meta.mojang.com/".to_string(), piston_meta);
    maps.insert("https://piston-data.mojang.com/".to_string(), piston_data);
    maps.insert(
        "https://resources.download.minecraft.net/".to_string(),
        resources,
    );
    maps.insert("https://libraries.minecraft.net/".to_string(), libraries);
    Mirror { maps }
}
pub fn ninecraft_mirror() -> Mirror {
    mirror(
        "https://launchermeta.9craft.ir".to_string(),
        "https://piston-meta.9craft.ir/".to_string(),
        "https://piston-data.9craft.ir/".to_string(),
        "https://resources-download.9craft.ir/".to_string(),
        "https://libraries.9craft.ir/".to_string(),
    )
}

pub fn mojang_mirror() -> Mirror {
    mirror(
        "https://launchermeta.mojang.com".to_string(),
        "https://piston-meta.mojang.com/".to_string(),
        "https://piston-meta.mojang.com/".to_string(),
        "https://resources.download.minecraft.net/".to_string(),
        "https://libraries.minecraft.net/".to_string(),
    )
}
