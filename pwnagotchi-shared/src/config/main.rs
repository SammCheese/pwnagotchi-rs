use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct MainConfig {
  pub name: Cow<'static, str>,
  pub mode: Cow<'static, str>,
  pub lang: Cow<'static, str>,
  pub iface: Cow<'static, str>,
  pub mon_start_cmd: Cow<'static, str>,
  pub mon_stop_cmd: Cow<'static, str>,
  pub mon_max_blind_epochs: u32,
  pub no_restart: bool,
  pub whitelist: Vec<Cow<'static, str>>,
  //confd
  //custom_plugin_repos
  pub plugins_path: Option<Cow<'static, str>>,
}

impl Default for MainConfig {
  fn default() -> Self {
    Self {
      name: "pwnagotchi".into(),
      mode: "auto".into(),
      lang: "en".into(),
      iface: "wlan0mon".into(),
      whitelist: vec![],
      mon_start_cmd: "/usr/bin/monstart".into(),
      mon_stop_cmd: "/usr/bin/monstop".into(),
      mon_max_blind_epochs: 5,
      no_restart: false,
      plugins_path: None,
    }
  }
}
