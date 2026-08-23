use crate::models::versions::VersionBase;
use crate::models::versions::VersionBase::{FABRIC, FORGE};
use crate::models::versions::VersionType;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Display;
use tauri::{AppHandle, Emitter};

#[derive(Deserialize, Debug)]
pub struct Manifest {
    pub latest: LatestVersionDetail,
    pub versions: Vec<VersionInfo>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    pub url: String,
}

// Model for reading individual asset entries inside the assets index file
#[derive(Deserialize, Debug)]
pub struct AssetObjects {
    pub objects: HashMap<String, AssetEntry>,
}

#[derive(Deserialize, Debug)]
pub struct AssetEntry {
    pub hash: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct LibraryInfo {
    pub name: String,
    pub size: u64,
    pub path: String,
    pub url: String,
}

pub struct LibraryRules {
    pub allowed_oses: Vec<String>,
    pub disallowed_oses: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoggingClient {
    pub argument: String,
    pub file: LoggingFile,
    #[serde(rename = "type")]
    pub _type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Logging {
    pub client: LoggingClient,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DownloadDetail {
    pub url: String,
    pub size: u64,
    pub sha1: String,
}

fn empty_object_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt_value: Option<Value> = Option::deserialize(deserializer)?;

    match opt_value {
        Some(Value::Object(map)) if map.is_empty() => Ok(None),
        Some(value) => T::deserialize(value)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftManifestVersion {
    pub libraries: Vec<Library>,
    pub asset_index: Option<AssetIndex>,
    pub downloads: Option<HashMap<String, DownloadDetail>>,
    #[serde(default, deserialize_with = "empty_object_as_none")]
    pub logging: Option<Logging>,
    pub java_version: Option<JavaVersion>,
    pub inherits_from: Option<String>,
    pub id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Debug, Deserialize)]
pub struct RuleOS {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<RuleOS>,
}

#[derive(Debug, Deserialize)]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    pub rules: Option<Vec<Rule>>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryArtifact>,
    pub classifiers: Option<HashMap<String, LibraryArtifact>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LibraryArtifact {
    pub path: Option<String>,
    pub url: String,
    pub size: Option<u64>,
    pub sha1: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LatestVersionDetail {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionLoader {
    pub id: String,
    pub base: VersionBase,
    pub date: String,
}

impl VersionLoader {
    pub fn get_installed_id(&self) -> String {
        match self.base {
            VersionBase::VANILLA => self.id.clone(),
            FORGE => {
                let id_clone = self.id.clone();
                let args = id_clone.split("-").collect::<Vec<_>>();
                let vanilla_id = args[0];
                let forge_ver = args[1].split("-").last().unwrap();
                format!("{}-forge-{}", vanilla_id, forge_ver)
            }
            VersionBase::NEOFORGE => self.id.clone(),
            FABRIC => {
                let args = self.id.split("-").collect::<Vec<_>>();
                format!("fabric-loader-{}-{}", args[1], args[0])
            }
            VersionBase::LITELOADER => self.id.clone(),
        }
    }
    pub fn get_fabric_loader_id(&self) -> String {
        self.id.split("-").collect::<Vec<&str>>()[1].to_string()
    }
    pub fn get_fabric_version_id(&self) -> String {
        self.id.split("-").collect::<Vec<&str>>()[0].to_string()
    }
}

// Models designed specifically for legacy Forge installer profile JSON extraction
#[derive(Deserialize, Debug)]
pub struct ForgeInstallProfile {
    pub install: Option<ForgeInstallData>,
    #[serde(rename = "versionInfo")]
    pub version_info: Option<ForgeVersionJsonInfo>,
    pub libraries: Option<Vec<ForgeLibrary>>,
}
fn default_mirror_list() -> String {
    "https://files.minecraftforge.net/mirror-brand.list".to_string()
}
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ForgeInstallData {
    pub profile_name: String,
    pub target: String,
    pub path: String,
    pub version: String,
    pub file_path: String,
    pub welcome: Option<String>,
    pub minecraft: String,

    #[serde(default = "default_mirror_list")]
    pub mirror_list: String,
    pub logo: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ForgeVersionJsonInfo {
    pub id: String,
    pub time: Option<String>,
    pub release_time: Option<String>,
    pub r#type: Option<String>,
    pub main_class: Option<String>,
    pub minecraft_arguments: Option<String>,
    pub minimum_launcher_version: Option<u32>,
    pub assets: Option<String>,
    pub inherits_from: Option<String>,
    pub jar: Option<String>,
    pub libraries: Vec<ForgeLibrary>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ForgeLibrary {
    pub name: String,
    pub url: Option<String>,
    pub downloads: Option<ForgeLibraryDownloads>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ForgeLibraryDownloads {
    pub artifact: Option<ForgeArtifact>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ForgeArtifact {
    pub path: Option<String>,
    pub url: String,
    pub size: Option<u64>,
    pub sha1: Option<String>,
}

// helper converter to adapter from Library struct
pub fn library_from_value_legacy(value: &Value) -> LibraryInfo {
    let library_name = value
        .get("name")
        .expect("Parsing library_name failed")
        .as_str()
        .expect("Parsing library_name failed");

    let library_downloads = value.get("downloads").unwrap();

    let library_artifact = library_downloads
        .get("artifact")
        .expect("Parsing library_downloads failed");

    let library_path = if library_artifact.get("path").is_none() {
        let args = library_name.split(":").collect::<Vec<&str>>();

        let group_id = args[0].replace(".", "/");

        let artifact = args[1];

        let version = args[2];

        let artifact_version = format!("{artifact}-{version}.jar");

        format!("{group_id}/{artifact}/{version}/{artifact_version}")
    } else {
        library_artifact["path"].as_str().unwrap().to_string()
    };

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStage {
    Manifest,
    Java,
    Libraries,
    Client,
    Assets,
    Logging,
    ForgeInstaller,
    FabricInstaller,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub stage: DownloadStage,
    pub stage_name: String,
    pub current_file: usize,
    pub total_files: usize,
    pub current_bytes: u64,
    pub total_bytes: u64,
    pub file_name: String,
    pub global_percentage: f32,
    pub stage_percentage: f32,
}

#[derive(Clone)]
struct StageSpan {
    pub start: f32,
    pub end: f32,
}

#[derive(Clone)]
pub struct PipelineProgressTracker {
    pub app_handle: AppHandle,
    pub stages: HashMap<DownloadStage, StageSpan>,
    pub current_stage: DownloadStage,
    pub current_file: usize,
    pub total_files: usize,
}

impl PipelineProgressTracker {
    pub fn new(app_handle: AppHandle, stage_weights: &[(DownloadStage, f32)]) -> Self {
        let total_weight: f32 = stage_weights.iter().map(|(_, w)| w).sum();
        let mut stages = HashMap::new();
        let mut accumulated = 0.0;

        for (stage, weight) in stage_weights {
            let portion = (weight / total_weight) * 100.0;
            stages.insert(
                *stage,
                StageSpan {
                    start: accumulated,
                    end: accumulated + portion,
                },
            );
            accumulated += portion;
        }

        let initial_stage = stage_weights
            .first()
            .map(|(s, _)| *s)
            .unwrap_or(DownloadStage::Done);

        Self {
            app_handle,
            stages,
            current_stage: initial_stage,
            current_file: 0,
            total_files: 1,
        }
    }

    pub fn start_stage(&mut self, stage: DownloadStage, total_files: usize) {
        self.current_stage = stage;
        self.total_files = total_files.max(1);
        self.current_file = 0;
        self.report("", 0, 0);
    }

    pub fn next_file(&mut self) {
        self.current_file = (self.current_file + 1).min(self.total_files);
    }

    pub fn report(&self, file_name: &str, current_bytes: u64, total_bytes: u64) {
        let span = self
            .stages
            .get(&self.current_stage)
            .cloned()
            .unwrap_or(StageSpan {
                start: 0.0,
                end: 100.0,
            });

        let file_ratio = (self.current_file as f32) / (self.total_files as f32);
        let chunk_ratio = if total_bytes > 0 {
            (current_bytes as f32 / total_bytes as f32) * (1.0 / self.total_files as f32)
        } else {
            0.0
        };

        let stage_progress_ratio = (file_ratio + chunk_ratio).clamp(0.0, 1.0);
        let stage_percentage = stage_progress_ratio * 100.0;

        let global_percentage = span.start + (stage_progress_ratio * (span.end - span.start));

        let stage_name = match self.current_stage {
            DownloadStage::Manifest => "Reading Manifest",
            DownloadStage::Java => "Downloading Java Runtime",
            DownloadStage::Libraries => "Downloading Libraries",
            DownloadStage::Client => "Downloading Game Client",
            DownloadStage::Assets => "Downloading Game Assets",
            DownloadStage::Logging => "Configuring Logging",
            DownloadStage::ForgeInstaller => "Installing Forge",
            DownloadStage::FabricInstaller => "Installing Fabric",
            DownloadStage::Done => "Completed",
        }
        .to_string();

        let progress = DownloadProgress {
            stage: self.current_stage,
            stage_name,
            current_file: self.current_file,
            total_files: self.total_files,
            current_bytes,
            total_bytes,
            file_name: file_name.to_string(),
            global_percentage: global_percentage.clamp(0.0, 100.0),
            stage_percentage: stage_percentage.clamp(0.0, 100.0),
        };

        let _ = self.app_handle.emit("download-progress", progress);
    }
}
