use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct DebugConfig {
  pub enabled: bool,
  pub last_session_file: Cow<'static, str>,
  pub identity_path: Cow<'static, str>,
  pub recovery_file: String,
}

impl Default for DebugConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      last_session_file: "/root/.pwnagotchi-last-session".into(),
      identity_path: "/etc/pwnagotchi".into(),
      recovery_file: "/root/.pwnagotchi-recovery".into(),
    }
  }
}
