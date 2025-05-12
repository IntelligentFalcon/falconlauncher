use serde_json::Value;

pub struct LatestVersionDetail {
    pub release: String,
    pub snapshot: String,
}

pub struct VersionInfo {
    pub id: String,
    pub version_type: VersionType,
    pub url: String,
}

pub enum VersionType {
    Beta,
    Alpha,
    Release,
    Snapshot,
}

pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub total_size: u64,
    pub url: String,
}

pub struct LibraryInfo {
    pub name: String,
    pub size: u64,
    pub path: String,
    pub url: String,
}
pub fn library_from_value(value: &Value) -> LibraryInfo {
    let library_name = value
        .get("name")
        .expect("Parsing library_name failed")
        .as_str()
        .expect("Parsing library_name failed");
    let library_downloads = value.get("downloads").unwrap();
    let library_artifact = library_downloads
        .get("artifact")
        .expect("Parsing library_downloads failed");

    let library_path = library_artifact
        .get("path")
        .expect("Parsing library path failed")
        .as_str()
        .expect("Parsing library path failed");
    let library_url = library_artifact
        .get("url")
        .expect("Parsing library_url failed")
        .as_str();
    let library_size = library_artifact
        .get("size")
        .expect("Parsing library_size failed")
        .as_u64()
        .expect("Parsing library_size failed");
    LibraryInfo {
        name: library_name.to_string(),
        size: library_size,
        path: library_path.to_string(),
        url: library_url.unwrap().to_string(),
    }
}
pub enum OperatingSystem {
    Windows,
    Linux,
    MacOS,
}

pub fn parse_os(os: String) -> String {
    os.to_lowercase().replace("darwin", "osx")
}
pub struct LibraryRules {
    pub allowed_oses: Vec<String>,
    pub disallowed_oses: Vec<String>,
}
