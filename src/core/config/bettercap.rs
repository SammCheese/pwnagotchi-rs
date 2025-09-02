use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct BettercapConfig {
  pub hostname: Cow<'static, str>,
  pub port: u16,
  pub username: Cow<'static, str>,
  pub password: Cow<'static, str>,
  pub silence: Vec<Cow<'static, str>>,
  pub handshakes: Cow<'static, str>,
}

impl Default for BettercapConfig {
  fn default() -> Self {
    Self {
      hostname: Cow::Borrowed("localhost"),
      port: 8081,
      username: Cow::Borrowed("user"),
      password: Cow::Borrowed("pass"),
      silence: Vec::new(),
      handshakes: Cow::Borrowed("handshakes"),
    }
  }
}
