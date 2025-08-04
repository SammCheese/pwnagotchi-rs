use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
  pub main: MainConfig,

  #[serde(flatten)]
  pub bettercap: BettercapConfig,

  #[serde(flatten)]
  pub plugins: HashMap<String, PluginConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MainConfig {
  pub name: String,
  pub mode: String,
  pub wifi_interface: String,
  pub whitelist: Vec<String>,
  
  #[serde(default = "default_bettercap_path")]
  pub bettercap_path: String,
  #[serde(default = "default_handshakes_path")]
  pub handshakes_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BettercapConfig {
    hostname: String,
    port: u16,
    username: String,
    password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PluginConfig<T = HashMap<String, String>> {
  pub enabled: bool,
  pub config: Option<T>,
}

fn default_handshakes_path() -> String {
  "/home/pi/handshakes".into()
}

fn default_bettercap_path() -> String {
  "/usr/bin/bettercap".into()
}

impl Default for Config {
  fn default() -> Self {
    Config {
      main: MainConfig {
        name: "pwnagotchi".into(),
        mode: "auto".into(),
        wifi_interface: "wlan0mon".into(),
        whitelist: vec![],
        bettercap_path: default_bettercap_path(),
        handshakes_path: default_handshakes_path(),
      },
      bettercap: BettercapConfig {
        hostname: "localhost".into(),
        port: 8081,
        username: "user".into(),
        password: "pass".into(),
      },
      plugins: HashMap::new(),
    }
  }
}

impl Config {
  pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
    let config_str = std::fs::read_to_string(path)
      .map_err(|e| format!("Failed to read config file: {}", e))?;

    toml::from_str(&config_str)
      .map_err(|e| format!("Failed to parse config file: {}", e))
  }

  pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
    let config_str = toml::to_string(self)
      .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(path, config_str)
      .map_err(|e| format!("Failed to write config file: {}", e))
      .map(|_| ())
  }

  pub fn get_config<T: std::str::FromStr>(&self, key: &str, default: T) -> T {
      let value = serde_json::to_value(self)
          .ok()
          .and_then(|v| Self::get_nested_value(&v, key).cloned());

      value
          .and_then(|v| Self::stringify_value(&v).parse::<T>().ok())
          .unwrap_or(default)
  }

  pub fn set_config(&mut self, key: &str, value: String) {
    if let Some(plugin) = self.plugins.get_mut(key) {
      if plugin.config.is_none() {
        plugin.config = Some(HashMap::new());
      }
      if let Some(config_map) = plugin.config.as_mut() {
        config_map.insert("value".to_string(), value);
      }
    } else {
      let mut config_map = HashMap::new();
      config_map.insert("value".to_string(), value);
      self.plugins.insert(key.to_string(), PluginConfig { enabled: true, config: Some(config_map) });
    }
  }

  fn get_nested_value<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
      path.split('.').fold(Some(value), |acc, key| {
          acc.and_then(|v| v.get(key))
      })
  }

  fn stringify_value(value: &Value) -> String {
      match value {
          Value::Null => "null".to_string(),
          Value::Bool(b) => b.to_string(),
          Value::Number(n) => n.to_string(),
          Value::String(s) => s.clone(),
          Value::Array(arr) => format!("{:?}", arr),
          Value::Object(obj) => format!("{:?}", obj),
      }
   }
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    Config::load("config.toml").unwrap_or_default()
});