use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct McModInfo {
    #[serde(rename = "modid")]
    pub mod_id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "logoFile")]
    pub logo_file: Option<String>,
    pub url: String,
    pub mcversion: String,
    pub version: String,
    pub screenshots: Vec<String>,
    pub dependencies: Vec<String>,
    #[serde(rename = "authorList")]
    pub author_list: Vec<String>,
    #[serde(rename = "updateUrl")]
    pub update_url: Option<String>,
    pub credits: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]

pub struct FabricModInfo {
    #[serde(rename = "id")]
    pub mod_id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "icon")]
    pub logo_file: Option<String>,
    pub contact: Option<FabricModInfoContact>,
    pub version: String,
    #[serde(rename = "authors")]
    pub author_list: Vec<String>,
    #[serde(rename = "updateUrl")]
    pub update_url: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]

pub struct FabricModInfoContact {
    pub homepage: Option<String>,
    pub issues: Option<String>,
    pub sources: Option<String>,
    pub twitter: Option<String>,
    pub discord: Option<String>,
}
