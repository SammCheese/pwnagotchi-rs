use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct PersonalityConfig {
  pub advertise: bool,
  pub deauth: bool,
  pub associate: bool,
  pub channels: Vec<u8>,
  pub min_rssi: i16,
  pub ap_ttl: u64,
  pub sta_ttl: u64,
  pub recon_time: u64,
  pub max_inactive_scale: u64,
  pub recon_inactive_multiplier: u64,
  pub hop_recon_time: u64,
  pub min_recon_time: u64,
  pub max_interactions: u64,
  pub max_misses_for_recon: u64,
  pub excited_num_epochs: u64,
  pub bored_num_epochs: u64,
  pub sad_num_epochs: u64,
  pub bond_encounters_factor: u64,
  pub throttle_a: f64,
  pub throttle_d: f64,
}

impl Default for PersonalityConfig {
  fn default() -> Self {
    Self {
      advertise: true,
      deauth: true,
      associate: true,
      channels: vec![],
      min_rssi: -200,
      ap_ttl: 120,
      sta_ttl: 300,
      recon_time: 30,
      max_inactive_scale: 2,
      recon_inactive_multiplier: 2,
      hop_recon_time: 10,
      min_recon_time: 5,
      max_interactions: 3,
      max_misses_for_recon: 5,
      excited_num_epochs: 10,
      bored_num_epochs: 15,
      sad_num_epochs: 25,
      bond_encounters_factor: 20000,
      throttle_a: 0.4,
      throttle_d: 0.9,
    }
  }
}