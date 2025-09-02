use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct IdentityConfig {
  pub path: Cow<'static, str>,
}

impl Default for IdentityConfig {
  fn default() -> Self {
    Self {
      path: Cow::Borrowed("/etc/.ssh/pwnagotchi/"),
    }
  }
}
