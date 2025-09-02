use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct MainConfig {
  pub name: Cow<'static, str>,
  pub mode: Cow<'static, str>,
  pub interface: Cow<'static, str>,
  pub mon_start_cmd: Cow<'static, str>,
  pub mon_stop_cmd: Cow<'static, str>,
  pub whitelist: Vec<Cow<'static, str>>,
  pub bettercap_path: Cow<'static, str>,
  pub handshakes_path: Cow<'static, str>,
  pub no_restart: bool,
  pub loglevel: Cow<'static, str>,
  pub log_path: Cow<'static, str>,
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
      handshakes_path: format!("{home}/handshakes").into(),
      mon_start_cmd: "/usr/bin/monstart".into(),
      mon_stop_cmd: "/usr/bin/monstop".into(),
      no_restart: false,
      loglevel: "info".into(),
      log_path: format!("{home}/logs/pwnagotchi.log").into(),
    }
  }
}
