use serde_json::Value;
use std::collections::HashMap;

use crate::core::models::net::AccessPoint;

#[derive(serde::Deserialize, Debug)]
pub struct BettercapSession {
  pub version: String,
  pub os: String,
  pub arch: String,
  pub goversion: String,
  pub resources: BResources,
  pub interfaces: Vec<BInterfaces>,
  pub options: HashMap<String, Value>,
  pub interface: BInterface,
  pub gateway: BInterface,
  pub env: BEnv,
  pub lan: BLan,
  pub wifi: BWifi,
  pub ble: BGeneric,
  pub hid: BGeneric,
  pub can: BGeneric,
  pub packets: BPackets,
  pub started_at: String,
  pub polled_at: String,
  pub active: bool,
  pub gps: BGps,
  pub modules: Vec<BModule>,
  pub caplets: Vec<BCaplets>,
}

#[derive(serde::Deserialize, Debug)]
pub struct BGps {
  #[serde(rename = "Updated")]
  pub updated: String,
  #[serde(rename = "Latitude")]
  pub latitude: f64,
  #[serde(rename = "Longitude")]
  pub longitude: f64,
  #[serde(rename = "FixQuality")]
  pub fix_quality: String,
  #[serde(rename = "NumSatellites")]
  pub num_satellites: u32,
  #[serde(rename = "HDOP")]
  pub hdop: f64,
  #[serde(rename = "Altitude")]
  pub altitude: f64,
  #[serde(rename = "Separation")]
  pub separation: f64,
}

#[derive(serde::Deserialize, Debug)]
pub struct BPackets {
  pub stats: HashMap<String, Value>,
  pub protos: HashMap<String, Value>,
  pub traffic: HashMap<String, BTraffic>,
}

#[derive(serde::Deserialize, Debug)]
pub struct BTraffic {
  pub sent: u64,
  pub received: u64,
}

#[derive(serde::Deserialize, Debug)]
pub struct BLan {
  pub hosts: Vec<BInterface>,
}

#[derive(serde::Deserialize, Debug)]
pub struct BGeneric {
  pub devices: Vec<BInterface>,
}

#[derive(serde::Deserialize, Debug)]
pub struct BEnv {
  pub data: HashMap<String, Value>,
}

#[derive(serde::Deserialize, Debug)]
pub struct BInterface {
  pub ipv4: String,
  pub ipv6: String,
  pub mac: String,
  pub hostname: String,
  pub alias: String,
  pub vendor: String,
  pub first_seen: String,
  pub last_seen: String,
  pub meta: Meta,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Meta {
  pub values: HashMap<String, Value>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct BResources {
  pub cpus: u32,
  pub max_cpus: u32,
  pub goroutines: u32,
  pub alloc: u64,
  pub sys: u64,
  pub gcs: u64,
}

#[derive(serde::Deserialize, Debug)]
pub struct BCaplets {
  pub path: String,
  pub size: u64,
  pub code: Vec<String>,
  pub name: String,
  pub scripts: Vec<BScripts>,
}

#[derive(serde::Deserialize, Debug)]
pub struct BScripts {
  pub path: String,
  pub size: u64,
  pub code: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct BWifi {
  pub aps: Vec<AccessPoint>,
}

#[derive(serde::Deserialize, Debug)]
pub struct BModule {
  pub name: String,
  pub description: String,
  pub author: String,
  pub parameters: HashMap<String, BModuleParameters>,
  pub handlers: Vec<BModuleHandler>,
  pub running: bool,
  pub state: HashMap<String, Value>,
}

#[derive(serde::Deserialize, Debug)]
pub struct BModuleHandler {
  pub name: String,
  pub description: String,
  pub parser: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct BModuleParameters {
  pub name: String,
  pub r#type: u16,
  pub description: String,
  pub default_value: String,
  pub current_value: String,
  pub validator: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct BInterfaces {
  pub index: u32,
  pub mtu: u32,
  pub name: String,
  pub mac: String,
  pub vendor: String,
  pub flags: Vec<String>,
  pub addresses: Vec<BInterfaceAddress>,
}

#[derive(serde::Deserialize, Debug)]
pub struct BInterfaceAddress {
  pub address: String,
  pub r#type: String,
}
