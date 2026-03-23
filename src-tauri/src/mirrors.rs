use std::collections::HashMap;
use std::iter::Map;

pub struct Mirror {
    pub maps: HashMap<String, String>,
}
impl Mirror {
    pub fn parse_url(&self, url: &String) -> String{
        let mut url = url.to_lowercase();
        url = url.replace("http://","https://");
        let domain = url.strip_prefix("https://").unwrap().split("/").next().unwrap();
        let https_domain = format!("https://{domain}/");
        println!("{}", url);
        println!("{}", https_domain);
        println!("{}", self.maps[https_domain.as_str()].as_str());
        if self.maps.contains_key(https_domain.as_str()) {
            println!("{}", url.replace(https_domain.as_str(), &*self.maps[&https_domain]));
            url.replace(https_domain.as_str(), &*self.maps[&https_domain])

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
    maps.insert("https://launchermeta.mojang.com/".to_string(), launcher_meta);
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
        "https://launchermeta.9craft.ir/".to_string(),
        "https://piston-meta.9craft.ir/".to_string(),
        "https://piston-data.9craft.ir/".to_string(),
        "https://resources-download.9craft.ir/".to_string(),
        "https://libraries-minecraft.9craft.ir/".to_string(),
    )
}

pub fn mojang_mirror() -> Mirror {
    mirror(
        "https://launchermeta.mojang.com/".to_string(),
        "https://piston-meta.mojang.com/".to_string(),
        "https://piston-meta.mojang.com/".to_string(),
        "https://resources.download.minecraft.net/".to_string(),
        "https://libraries.minecraft.net/".to_string(),
    )
}
