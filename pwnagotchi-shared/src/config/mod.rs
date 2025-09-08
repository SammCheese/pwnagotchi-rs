#![allow(clippy::missing_errors_doc)]

mod bettercap;
mod debug;
mod faces;
mod fs;
mod main;
mod personality;
mod plugins;
mod ui;

use std::{collections::HashMap, fmt::Display, path::Path, sync::OnceLock};

pub use bettercap::BettercapConfig;
pub use debug::DebugConfig;
pub use faces::FaceConfig;
pub use fs::FSConfig;
pub use main::MainConfig;
pub use personality::PersonalityConfig;
pub use plugins::PluginConfig;
use serde::{Deserialize, Serialize};
pub use ui::UIConfig;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct Config {
  pub main: MainConfig,
  pub bettercap: BettercapConfig,
  pub plugins: HashMap<String, PluginConfig>,
  pub personality: PersonalityConfig,
  pub fs: FSConfig,
  pub ui: UIConfig,
  pub faces: FaceConfig,
  pub debug: DebugConfig,
}

impl Display for Config {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", toml::to_string(self).unwrap_or_default())
  }
}

impl Config {
  pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
    let config_str =
      std::fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {e}"))?;
    let config: Self =
      toml::from_str(&config_str).map_err(|e| format!("Failed to parse config file: {e}"))?;
    Ok(config)
  }

  pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
    let config_str =
      toml::to_string(self).map_err(|e| format!("Failed to serialize config: {e}"))?;
    std::fs::write(path, config_str).map_err(|e| format!("Failed to write config file: {e}"))?;
    Ok(())
  }
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn init_config<P: AsRef<std::path::Path>>(path: P) {
  let config = Config::load(path).unwrap_or_default();
  let _ = CONFIG.set(config);
}

/// Returns a reference to the global configuration.
///
/// # Panics
/// Panics if the configuration has not been initialized.
pub fn config() -> &'static Config {
  #[allow(clippy::expect_used)]
  CONFIG.get().expect("Config not initialized")
}
