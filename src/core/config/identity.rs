use serde::{ Deserialize, Serialize };

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct IdentityConfig {
  pub path: String,
}

impl Default for IdentityConfig {
  fn default() -> Self {
    Self {
      path: "/etc/.ssh/pwnagotchi/".to_owned(),
    }
  }
}
