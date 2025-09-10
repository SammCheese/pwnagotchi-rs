use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct LogConfig {
  pub path: Cow<'static, str>,
  pub path_debug: Cow<'static, str>,
  pub rotation: LogRotationConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct LogRotationConfig {
  pub enabled: bool,
  pub size: Cow<'static, str>,
}

impl Default for LogRotationConfig {
  fn default() -> Self {
    Self { enabled: true, size: "10M".into() }
  }
}

impl Default for LogConfig {
  fn default() -> Self {
    Self {
      path: "/etc/pwnagotchi/log/pwnagotchi.log".into(),
      path_debug: "/etc/pwnagotchi/log/pwnagotchi_debug.log".into(),
      rotation: LogRotationConfig::default(),
    }
  }
}
