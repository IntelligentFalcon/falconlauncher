use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatusType {
    Approved,
    Archived,
    Rejected,
    Draft,
    Unlisted,
    Processing,
    Withheld,
    Scheduled,
    Private,
    Unknown
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Mod,
    Modpack,
    Resourcepack,
    Shader,
    Plugin,
    Datapack
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentType {
    ClientAndServer,
    ClientOnly,
    ClientOnlyServerOptional,
    SingleplayerOnly,
    ServerOnly,
    ServerOnlyClientOptional,
    DedicatedServerOnly,
    ClientOrServer,
    ClientOrServerPrefersBoth,
    Unknown
}
#[derive(Debug, Serialize, Deserialize, PartialEq,Clone)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum MonetizationStatusType {
    Monetized,
    Demonetized,
    ForceDemonetized
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum FileType {
    RequiredResourcePack,
    OptionalResourcePack,
    SourcesJar,
    DevJar,
    JavadocJar,
    Unknown,
    Signature
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VersionType {
    Release,
    Beta,
    Alpha
}


#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VersionStatusType {
    Listed,
    Archived,
    Draft,
    Unlisted,
    Scheduled,
    Unknown
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DisclosureType {
    AiContent,
    AiContentCode,
    AiContentAssets,
    AiContentText,
    AiContentFunctionality,
    Advertisements,
    EpilepsyTriggers,
    SystemInteractions,
    Telemetry,
    TelemetryOptIn,
    TelemetryOptOut,
    TelemetryAlwaysActive,
    DerivativeWork,
    PaidFeatures,
    Archived
}


/// More info on https://docs.modrinth.com/api/operations/searchprojects/
#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthSearchResult {
    pub hits: Vec<ModrinthSearchResultMod>,
    /// The number of results that were skipped by the query
    pub offset: usize,
    /// The number of results that were returned by the query
    pub limit: usize,
    /// The total number of results that match the query
    pub total_hits: usize

}
/// More info on https://docs.modrinth.com/api/operations/searchprojects/
#[derive(Serialize, Deserialize, Debug)]
pub struct ModrrinthSearchResultError {
    pub error: String,
    pub description: String,
}

/// More info on https://docs.modrinth.com/api/operations/getproject/
#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthSearchResultMod {
    pub project_id: String,
    pub project_type: ProjectType,
    pub all_project_types: Vec<String>,
    pub title: String,
    pub description: String,
    pub author: String,
    pub categories: Vec<String>,
    pub display_categories: Vec<String>,
    pub versions: Vec<String>,
    pub downloads: usize,
    pub follows: usize,
    pub icon_url: String,
    pub date_created: String,
    pub date_modified: String,
    pub latest_version: String,
    pub license: String,
    pub environment: Vec<EnvironmentType>,
    pub disclosure_types: Vec<DisclosureType>,
    pub gallery: Vec<String>,

    pub slug: Option<String>,
    pub author_id: Option<String>,
    pub organization: Option<String>,
    pub organization_id: Option<String>,
    pub featured_gallery: Option<String>,
    pub color: Option<i64>,

    // Kept as Option because docs say they are deprecated and might be removed soon
    pub client_side: Option<String>,
    pub server_side: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthVersionDependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: DependencyType
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthHashes {
    pub sha1: Option<String>,
    pub sha512: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthFile {
    pub hashes: ModrinthHashes,
    pub url: String,
    #[serde(rename = "filename")]
    pub file_name: String,
    pub primary: bool,
    pub size: usize,
    pub file_type: Option<FileType>
}


/// more info on https://docs.modrinth.com/api/operations/getversion/
#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthVersion {
    pub name: Option<String>,
    pub version_number: Option<String>,
    pub changelog: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<ModrinthVersionDependency>,
    #[serde(default)]
    pub game_versions: Vec<String>,
    pub version_type: Option<VersionType>,
    #[serde(default)]
    pub loaders: Vec<String>,

    pub featured: Option<bool>,
    pub status: Option<VersionStatusType>,
    pub requested_status: Option<VersionStatusType>,
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub date_published: String,
    pub downloads: usize,
    pub changelog_url: Option<String>,
    pub environment: EnvironmentType,
    pub files: Vec<ModrinthFile>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthLicense {
    pub id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthDonationUrl {
    pub id: Option<String>,
    pub platform: Option<String>,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthGalleryImage {
    pub url: String,
    #[serde(default)]
    pub featured: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created: Option<String>,
    pub ordering: Option<i32>,
}

/// More info on https://docs.modrinth.com/api/operations/getproject/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthMod {
    pub id: String,
    pub title: String,
    pub description: String,
    pub project_type: ProjectType,

    pub team: Option<String>,
    pub body: Option<String>,
    pub status: Option<ProjectStatusType>,

    #[serde(default)] pub categories: Vec<String>,
    #[serde(default)] pub additional_categories: Vec<String>,
    #[serde(default)] pub environment: Vec<EnvironmentType>,
    #[serde(default)] pub game_versions: Vec<String>,
    #[serde(default)] pub loaders: Vec<String>,
    #[serde(default)] pub versions: Vec<String>,

    pub license: Option<ModrinthLicense>,
    #[serde(default)] pub gallery: Vec<ModrinthGalleryImage>,
    #[serde(default)] pub donation_urls: Vec<ModrinthDonationUrl>,

    pub published: Option<String>,
    pub updated: Option<String>,

    #[serde(default)] pub downloads: usize,
    #[serde(default)] pub followers: usize,

    pub monetization_status: Option<MonetizationStatusType>,
    pub slug: Option<String>,
    pub organization: Option<String>,

    pub requested_status: Option<ProjectStatusType>,

    pub approved: Option<String>,
    pub queued: Option<String>,
    pub icon_url: Option<String>,
    pub raw_icon_url: Option<String>,
    pub color: Option<i64>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,

    pub client_side: Option<String>,
    pub server_side: Option<String>,
    pub body_url: Option<String>,
    pub moderator_message: Option<String>,
}

