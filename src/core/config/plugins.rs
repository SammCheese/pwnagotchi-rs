use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct PluginConfig {
  pub enabled: bool,
  pub config: Option<serde_json::Value>,
}

impl Default for PluginConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      config: Some(serde_json::json!({})),
    }
  }
}