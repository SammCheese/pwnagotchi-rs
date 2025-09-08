use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct MainConfig {
  pub name: Cow<'static, str>,
  pub mode: Cow<'static, str>,
  pub lang: Cow<'static, str>,
  pub interface: Cow<'static, str>,
  pub mon_start_cmd: Cow<'static, str>,
  pub mon_stop_cmd: Cow<'static, str>,
  pub whitelist: Vec<Cow<'static, str>>,
  pub bettercap_path: Cow<'static, str>,
  pub no_restart: bool,
  pub log: Cow<'static, str>,
}

impl Default for MainConfig {
  fn default() -> Self {
    Self {
      name: "pwnagotchi".into(),
      mode: "auto".into(),
      lang: "en".into(),
      interface: "wlan0mon".into(),
      whitelist: vec![],
      bettercap_path: "/usr/bin/bettercap".into(),
      mon_start_cmd: "/usr/bin/monstart".into(),
      mon_stop_cmd: "/usr/bin/monstop".into(),
      no_restart: false,
      log: "/etc/pwnagotchi/log/pwnagotchi.log".into(),
    }
  }
}
