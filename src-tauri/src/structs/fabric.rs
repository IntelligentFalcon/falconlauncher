use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize,Clone   )]

pub struct FabricLoader {
    pub separator: String,
    pub build: u16,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

#[derive(Serialize, Deserialize, Clone,Debug)]
pub struct FabricMinecraftVersion {
    pub version: String,
    pub stable: bool,
}

#[derive(Serialize, Deserialize)]
pub struct FabricInstaller {
    pub url: String,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}


