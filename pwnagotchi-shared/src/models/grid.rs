use crate::config::PersonalityConfig;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Advertisement {
  pub name: String,
  pub version: String,
  pub identity: String,
  pub face: String,
  pub pwnd_run: u32,
  pub pwnd_total: u32,
  pub uptime: u32,
  pub epoch: u32,
  pub policy: PersonalityConfig,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct PeerResponse {
  pub fingerprint: Option<String>,
  pub met_at: Option<String>,
  pub encounters: Option<u32>,
  pub prev_seen_at: Option<String>,
  pub detected_at: Option<String>,
  pub seen_at: Option<String>,
  pub channel: Option<u8>,
  pub rssi: Option<i16>,
  pub session_id: Option<String>,
  pub advertisement: Option<Advertisement>,
}
