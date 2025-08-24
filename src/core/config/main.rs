use serde::{Deserialize, Serialize};

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
          mon_start_cmd: "ifconfig wlan0 down && iw dev wlan0 set type monitor && ifconfig wlan0 up".into(),
          mon_stop_cmd: "ifconfig wlan0 down && iw dev wlan0 set type managed && ifconfig wlan0 up".into(),
          no_restart: false,
        }
    }
}