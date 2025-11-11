use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::net::AccessPoint;

#[derive(Deserialize, Serialize, Debug)]
pub struct BettercapSession {
  pub version: Cow<'static, str>,
  pub os: Cow<'static, str>,
  pub arch: Cow<'static, str>,
  pub goversion: Cow<'static, str>,
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

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
pub struct BPackets {
  pub stats: HashMap<String, Value>,
  pub protos: HashMap<String, Value>,
  pub traffic: HashMap<String, BTraffic>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BTraffic {
  pub sent: u64,
  pub received: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BLan {
  pub hosts: Vec<BInterface>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BGeneric {
  pub devices: Vec<BInterface>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BEnv {
  pub data: HashMap<String, Value>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BInterface {
  pub ipv4: Cow<'static, str>,
  pub ipv6: Cow<'static, str>,
  pub mac: Cow<'static, str>,
  pub hostname: Cow<'static, str>,
  pub alias: Cow<'static, str>,
  pub vendor: Cow<'static, str>,
  pub first_seen: Cow<'static, str>,
  pub last_seen: Cow<'static, str>,
  pub meta: Meta,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Meta {
  pub values: HashMap<String, Value>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct BResources {
  pub cpus: u32,
  pub max_cpus: u32,
  pub goroutines: u32,
  pub alloc: u64,
  pub sys: u64,
  pub gcs: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BCaplets {
  pub path: Cow<'static, str>,
  pub size: u64,
  pub code: Vec<Cow<'static, str>>,
  pub name: Cow<'static, str>,
  pub scripts: Vec<BScripts>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BScripts {
  pub path: Cow<'static, str>,
  pub size: u64,
  pub code: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BWifi {
  pub aps: Vec<AccessPoint>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BModule {
  pub name: Cow<'static, str>,
  pub description: Cow<'static, str>,
  pub author: Cow<'static, str>,
  pub parameters: HashMap<String, BModuleParameters>,
  pub handlers: Vec<BModuleHandler>,
  pub running: bool,
  pub state: HashMap<String, Value>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BModuleHandler {
  pub name: Cow<'static, str>,
  pub description: Cow<'static, str>,
  pub parser: Cow<'static, str>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BModuleParameters {
  pub name: Cow<'static, str>,
  pub r#type: u16,
  pub description: Cow<'static, str>,
  pub default_value: Cow<'static, str>,
  pub current_value: Cow<'static, str>,
  pub validator: Cow<'static, str>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BInterfaces {
  pub index: u32,
  pub mtu: u32,
  pub name: Cow<'static, str>,
  pub mac: Cow<'static, str>,
  pub vendor: Cow<'static, str>,
  pub flags: Vec<Cow<'static, str>>,
  pub addresses: Vec<BInterfaceAddress>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BInterfaceAddress {
  pub address: Cow<'static, str>,
  pub r#type: Cow<'static, str>,
}
