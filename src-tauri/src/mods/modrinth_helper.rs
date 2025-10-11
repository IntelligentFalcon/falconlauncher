use std::fmt::format;
use serde::{Deserialize, Serialize};

pub async fn search_for_project(name: String, facets: String, index: String, offset: u64, limit: u64) -> ModrinthSearchResults{
    let api = format!("https://api.modrinth.com/v2/search?name={name}&facets={facets}&offset={offset}&limit={limit}&index={index}");
    println!("{}", api);
    let results = reqwest::get(&api).await.unwrap().json::<ModrinthSearchResults>().await.unwrap();
    results
}
pub async fn get_project(project_id: String) {
    let api = format!("https://api.modrinth.com/v2/project/{project_id}");
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthSearchResults {
    pub hits: Vec<ModrinthSearchResult>,
    pub offset: u64,
    pub limit: u64,
    pub total_hits: u64
}
#[derive(Serialize,Deserialize, Debug)]
pub struct ModrinthSearchResult {
    pub project_id: String,
    pub author: String,
    pub versions: Vec<String>,
    pub date_created: String,
    pub date_modified: String,
    pub license: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub categories: Option<Vec<String>>,
    pub project_type: String,
    pub downloads: u64,
    pub slug: Option<String>
}

pub struct SearchFacet {
    data: String,
}
impl SearchFacet {
    pub fn new() -> SearchFacet {
        SearchFacet {
            data: "".to_string(),
        }
    }

    pub fn version(mut self, version: &str) -> SearchFacet {
        self.data
            .push_str(format!("[\"versions:{}\"]", version).as_str());
        self
    }

    pub fn category(mut self, category: &str) -> SearchFacet {
        self.data
            .push_str(format!("[\"categories:{}\"]", category).as_str());
        self
    }
    pub fn project_type(mut self, project_type: &str) -> SearchFacet {
        self.data
            .push_str(format!("[\"project_types:{}\"]", project_type).as_str());
        self
    }
    pub fn get_str(&self) -> String {
        let mut s = self.data.clone();
        if s.len() == 0 {
            return "".to_string();
        }
        s.insert(0, '[');

        s.insert(&s.len() - 1, ']');

        s.replace("][", "],[")
    }
}
