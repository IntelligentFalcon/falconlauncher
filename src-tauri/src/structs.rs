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
    pub sha1: String,
    pub size: u64,
    pub path: String,
    pub uri: String,
}

pub enum OperatingSystem {
    Windows,
    Linux,
    MacOS,
}

pub fn parse_os(os: String) -> String {
    os.to_lowercase().replace("darwin", "osx")
}
