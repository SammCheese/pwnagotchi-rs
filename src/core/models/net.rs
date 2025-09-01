use std::collections::HashMap;

use crate::core::{mesh::advertiser::Advertisement, models::bettercap::Meta};

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Station {
  pub ipv4: String,
  pub ipv6: String,
  pub mac: String,
  pub hostname: String,
  pub alias: String,
  pub vendor: String,
  pub first_seen: String,
  pub last_seen: String,
  pub meta: Meta,
  pub frequency: u32,
  pub channel: u8,
  pub rssi: i32,
  pub sent: u32,
  pub received: u32,
  pub encryption: String,
  pub cipher: String,
  pub authentication: String,
  pub wps: HashMap<String, String>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct AccessPoint {
  pub ipv4: String,
  pub ipv6: String,
  pub mac: String,
  pub hostname: String,
  pub alias: String,
  pub vendor: String,
  pub first_seen: String,
  pub last_seen: String,
  pub meta: Meta,
  pub frequency: u32,
  pub channel: u8,
  pub rssi: i32,
  pub sent: u32,
  pub received: u32,
  pub encryption: String,
  pub cipher: String,
  pub authentication: String,
  pub wps: HashMap<String, String>,
  pub clients: Vec<Station>,
  pub handshake: bool,
}

#[derive(Debug, Clone)]
pub struct Peer {
  pub session_id: String,
  pub channel: u8,
  pub rssi: i32,
  pub identity: String,
  pub advertisement: Advertisement,
}

#[derive(Debug, Clone)]
pub struct Handshake {
  pub mac: String,
  pub timestamp: std::time::SystemTime,
  pub filename: String,
}
