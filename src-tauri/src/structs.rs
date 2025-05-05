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
    id: String,
    sha1: String,
    size: u64,
    total_size: u64,
    url: String,
}
