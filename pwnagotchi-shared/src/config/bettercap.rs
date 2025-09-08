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
    let silenced = vec![
      "ble.device.new",
      "ble.device.lost",
      "ble.device.service.discovered",
      "ble.device.characteristic.discovered",
      "ble.device.disconnected",
      "ble.device.connected",
      "ble.connection.timeout",
      "wifi.client.new",
      "wifi.client.lost",
      "wifi.client.probe",
      "wifi.ap.new",
      "wifi.ap.lost",
      "mod.started",
      "sys.log",
    ]
    .into_iter()
    .map(Cow::Borrowed)
    .collect();
    Self {
      hostname: Cow::Borrowed("127.0.0.1"),
      port: 8081,
      username: Cow::Borrowed("user"),
      password: Cow::Borrowed("pass"),
      silence: silenced,
      handshakes: Cow::Borrowed("/home/pi/handshakes"),
    }
  }
}
