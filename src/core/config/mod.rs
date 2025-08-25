#![allow(clippy::missing_errors_doc)]
mod main;
mod bettercap;
mod personality;
mod fs;
mod plugins;

pub use main::MainConfig;
pub use bettercap::BettercapConfig;
pub use personality::PersonalityConfig;
pub use fs::FSConfig;
pub use plugins::PluginConfig;

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};


#[derive(Debug, Deserialize, Serialize, Clone)]
#[derive(Default)]
pub struct Config {
  pub main: MainConfig,
  pub bettercap: BettercapConfig,
  pub plugins: HashMap<String, PluginConfig>,
  pub personality: PersonalityConfig,
  pub fs: FSConfig,
}


impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let config_str = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {e}"))?;
        toml::from_str(&config_str).map_err(|e| format!("Failed to parse config file: {e}"))
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let config_str =
            toml::to_string(self).map_err(|e| format!("Failed to serialize config: {e}"))?;
        std::fs::write(path, config_str)
            .map_err(|e| format!("Failed to write config file: {e}"))?;
        Ok(())
    }
}


pub static CONFIG: std::sync::LazyLock<Config> =
    std::sync::LazyLock::new(|| Config::load("config.toml").unwrap_or_default());