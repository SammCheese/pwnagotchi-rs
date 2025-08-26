use serde::{ Deserialize, Serialize };

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct MainConfig {
  pub name: String,
  pub mode: String,
  pub interface: String,
  pub mon_start_cmd: String,
  pub mon_stop_cmd: String,
  pub whitelist: Vec<String>,
  pub bettercap_path: String,
  pub handshakes_path: String,
  pub no_restart: bool,
  pub loglevel: String,
}

impl Default for MainConfig {
  fn default() -> Self {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    Self {
      name: "pwnagotchi".into(),
      mode: "auto".into(),
      interface: "wlan0mon".into(),
      whitelist: vec![],
      bettercap_path: "/usr/bin/bettercap".into(),
      handshakes_path: format!("{home}/handshakes"),
      mon_start_cmd: "/usr/bin/monstart".into(),
      mon_stop_cmd: "/usr/bin/monstop".into(),
      no_restart: false,
      loglevel: "info".into(),
    }
  }
}
