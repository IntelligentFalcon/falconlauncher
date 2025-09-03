use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct McModInfo {
    #[serde(rename = "modid")]
    pub mod_id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "logoFile")]
    pub logo_file: String,
    pub url: String,
    pub mcversion: String,
    pub version: String,
    pub screenshots: Vec<String>,
    pub dependencies: Vec<String>,
    #[serde(rename = "authorList")]
    pub author_list: Vec<String>,
    #[serde(rename = "updateUrl")]
    pub update_url: String,
    pub credits: String,


}