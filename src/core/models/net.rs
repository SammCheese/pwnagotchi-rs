use std::{borrow::Cow, collections::HashMap};

use crate::core::{mesh::advertiser::Advertisement, models::bettercap::Meta};

#[derive(serde::Deserialize, Debug, Clone)]

pub struct Station {
  pub ipv4: Cow<'static, str>,
  pub ipv6: Cow<'static, str>,
  pub mac: Cow<'static, str>,
  pub hostname: Cow<'static, str>,
  pub alias: Cow<'static, str>,
  pub vendor: Cow<'static, str>,
  pub first_seen: Cow<'static, str>,
  pub last_seen: Cow<'static, str>,
  pub meta: Meta,
  pub frequency: u32,
  pub channel: u8,
  pub rssi: i32,
  pub sent: u32,
  pub received: u32,
  pub encryption: Cow<'static, str>,
  pub cipher: Cow<'static, str>,
  pub authentication: Cow<'static, str>,
  pub wps: HashMap<String, String>,
}

#[derive(serde::Deserialize, Debug, Clone)]

pub struct AccessPoint {
  pub ipv4: Cow<'static, str>,
  pub ipv6: Cow<'static, str>,
  pub mac: Cow<'static, str>,
  pub hostname: Cow<'static, str>,
  pub alias: Cow<'static, str>,
  pub vendor: Cow<'static, str>,
  pub first_seen: Cow<'static, str>,
  pub last_seen: Cow<'static, str>,
  pub meta: Meta,
  pub frequency: u32,
  pub channel: u8,
  pub rssi: i32,
  pub sent: u32,
  pub received: u32,
  pub encryption: Cow<'static, str>,
  pub cipher: Cow<'static, str>,
  pub authentication: Cow<'static, str>,
  pub wps: HashMap<String, String>,
  pub clients: Vec<Station>,
  pub handshake: bool,
}

#[derive(Debug, Clone)]

pub struct Peer {
  pub session_id: Cow<'static, str>,
  pub channel: u8,
  pub rssi: i32,
  pub identity: Cow<'static, str>,
  pub advertisement: Advertisement,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Handshake {
  pub mac: String,
  pub timestamp: std::time::SystemTime,
  pub filename: String,
}

impl Default for Handshake {
  fn default() -> Self {
    Self {
      mac: String::default(),
      timestamp: std::time::SystemTime::UNIX_EPOCH,
      filename: String::default(),
    }
  }
}
