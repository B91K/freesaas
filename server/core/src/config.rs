//! `freesaas.yaml`: the single file an operator edits to wire up a deployment.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_listen")]
    pub listen: String,
    pub identity: ProviderConfig,
    pub storage: Vec<ProviderConfig>,
    #[serde(default)]
    pub ai: Option<ProviderConfig>,
}

/// `type` selects the implementation; every other key is left for that
/// implementation to parse.
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(flatten)]
    pub settings: serde_json::Map<String, serde_json::Value>,
}

impl ProviderConfig {
    pub fn str(&self, key: &str) -> Option<&str> {
        self.settings.get(key).and_then(|v| v.as_str())
    }

    /// The configured id, falling back to the provider type.
    pub fn id_or_kind(&self) -> &str {
        self.id.as_deref().unwrap_or(&self.kind)
    }
}

fn default_listen() -> String {
    "0.0.0.0:8080".to_string()
}
