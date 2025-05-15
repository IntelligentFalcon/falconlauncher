use crate::utils::load_json_url;
use serde_json::Value;

/// Returns the version_manifest.json file in a Value structure if the parse process was succeed.
pub async fn load_version_manifest() -> Option<Value> {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    load_json_url(&url.to_string()).await
}

