use serde_json::Value;
use serde::{Deserialize, Serialize};
use crate::models::versions::VersionType;
use crate::models::versions::VersionBase;
use crate::models::versions::VersionBase::{FABRIC, FORGE};
#[derive(Deserialize, Debug)]
pub struct Manifest {
    pub latest: LatestVersionDetail,
    pub versions: Vec<VersionInfo>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize")]
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

pub struct LibraryRules {
    pub allowed_oses: Vec<String>,
    pub disallowed_oses: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MinecraftManifestVersion {
    pub libraries: Value,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndex>,
    pub downloads: Option<Value>
}

#[derive(Debug, Deserialize)]
pub struct Library {
    pub name: String,
    pub path: String,
    pub downloads: LibraryDownloads
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

#[derive(Debug, Deserialize, Serialize)]
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
            /// FIX THESE LATER
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

#[derive(Debug, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryArtifact>,
    pub classifiers: Option<LibraryClassifier>,
}

#[derive(Debug, Deserialize)]
pub struct LibraryArtifact {
    pub path: Option<String>,
    pub url: String,
    pub size: u64,
}

#[derive(Debug, Deserialize)]
pub struct LibraryClassifier {

}