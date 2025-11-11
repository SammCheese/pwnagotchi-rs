#[derive(serde::Serialize, serde::Deserialize)]
pub struct GridPolicy {
  pub advertise: bool,
  pub ap_ttl: u64,
  pub associate: bool,
  pub bond_encounters_factor: u64,
  pub bored_num_epochs: u64,
  pub channels: Vec<u8>,
  pub deauth: bool,
  pub excited_num_epochs: u64,
  pub hop_recon_time: u64,
  pub max_inactive_scale: u8,
  pub max_interactions: u8,
  pub max_misses_for_recon: u8,
  pub min_recon_time: u64,
  pub min_rssi: i32,
  pub recon_inactive_multiplier: u8,
  pub recon_time: u64,
  pub sad_num_epochs: u64,
  pub sta_ttl: u64,
  pub throttle_a: f32,
  pub throttle_d: f32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GridSessionStats {
  pub associated: usize,
  pub avg_reward: f64,
  pub deauthed: usize,
  pub duration: Option<String>,
  pub epochs: usize,
  pub handshakes: usize,
  pub max_reward: f64,
  pub min_reward: f64,
  pub peers: usize,
  pub train_epochs: usize,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GridDataAdvertisement {
  pub epoch: u64,
  pub face: String,
  pub identity: String,
  pub name: String,
  pub policy: GridPolicy,
  pub pwnd_run: u64,
  pub pwnd_total: u64,
  pub uptime: u64,
  pub version: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GridDataResponse {
  pub ai: String,
  pub bettercap: String,
  pub build: String,
  pub language: String,
  pub opwngrid: String,
  pub plugins: Vec<String>,
  pub session: GridSessionStats,
  pub advertisement: GridDataAdvertisement,
  pub uname: String,
  pub version: String,
}
