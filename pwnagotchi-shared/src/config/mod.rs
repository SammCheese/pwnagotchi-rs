#![allow(clippy::missing_errors_doc)]

mod bettercap;
mod debug;
mod faces;
mod fs;
mod log;
mod main;
mod personality;
mod plugins;
mod ui;

use std::{
  collections::HashMap,
  fmt::Display,
  ops::{Deref, DerefMut},
  path::Path,
  sync::OnceLock,
};

pub use bettercap::BettercapConfig;
pub use debug::DebugConfig;
pub use faces::FaceConfig;
pub use fs::FSConfig;
pub use log::LogConfig;
pub use main::MainConfig;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
pub use personality::PersonalityConfig;
pub use plugins::PluginConfig;
use serde::{Deserialize, Serialize};
pub use ui::UIConfig;

use crate::logger::LOGGER;

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
  pub log: LogConfig,
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

pub fn save_current_config() -> Result<(), String> {
  let path = CONFIG_PATH.get().ok_or("config path not set")?;
  let guard = config_read();
  guard.save(path)
}

pub static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();
pub static CONFIG_PATH: OnceLock<String> = OnceLock::new();

pub fn init_config<P: AsRef<Path>>(path: P) {
  let cfg = Config::load(path.as_ref()).unwrap_or_default();
  CONFIG.get_or_init(|| RwLock::new(cfg));
  CONFIG_PATH.get_or_init(|| path.as_ref().to_string_lossy().to_string());
}

fn try_parse_config_from_args() -> Option<String> {
  let args: Vec<String> = std::env::args().collect();
  let mut args_iter = args.iter();
  while let Some(arg) = args_iter.next() {
    if (arg == "--config" || arg == "-C")
      && let Some(path) = args_iter.next()
    {
      return Some(path.clone());
    }
  }
  None
}

pub fn config_read() -> RwLockReadGuard<'static, Config> {
  CONFIG
    .get_or_init(|| {
      let path = try_parse_config_from_args().unwrap_or_else(|| "config.toml".to_string());
      RwLock::new(Config::load(path).unwrap_or_default())
    })
    .read()
}

pub fn config_write() -> ConfigWriteGuard<'static> {
  let guard = CONFIG
    .get_or_init(|| {
      let path = try_parse_config_from_args().unwrap_or_else(|| "config.toml".to_string());
      RwLock::new(Config::load(path).unwrap_or_default())
    })
    .write();

  ConfigWriteGuard { guard }
}

pub fn with_config_read<F, R>(f: F) -> R
where
  F: FnOnce(&Config) -> R,
{
  let guard = config_read();
  f(&guard)
}

pub fn with_config_write<F, R>(f: F) -> R
where
  F: FnOnce(&mut Config) -> R,
{
  let mut guard = config_write();
  f(&mut guard)
}

pub struct ConfigWriteGuard<'a> {
  guard: RwLockWriteGuard<'a, Config>,
}

impl<'a> Deref for ConfigWriteGuard<'a> {
  type Target = Config;

  fn deref(&self) -> &Self::Target {
    &self.guard
  }
}

impl<'a> DerefMut for ConfigWriteGuard<'a> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.guard
  }
}

impl<'a> Drop for ConfigWriteGuard<'a> {
  fn drop(&mut self) {
    if let Some(path) = CONFIG_PATH.get() {
      if let Err(e) = self.guard.save(path) {
        LOGGER.log_error("CONFIG", &format!("Failed to auto-save config: {}", e));
      }
    } else {
      LOGGER.log_error("CONFIG", "Failed to auto-save config: CONFIG_PATH not set.");
    }
  }
}
