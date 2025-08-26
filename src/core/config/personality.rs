use serde::{ Deserialize, Serialize };

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct PersonalityConfig {
  pub advertise: bool,
  pub deauth: bool,
  pub associate: bool,
  pub channels: Vec<u8>,
  pub min_rssi: i16,
  pub ap_ttl: u32,
  pub sta_ttl: u32,
  pub recon_time: u32,
  pub max_inactive_scale: u32,
  pub recon_inactive_multiplier: u32,
  pub hop_recon_time: u32,
  pub min_recon_time: u32,
  pub max_interactions: u32,
  pub max_misses_for_recon: u32,
  pub excited_num_epochs: u32,
  pub bored_num_epochs: u32,
  pub sad_num_epochs: u32,
  pub bond_encounters_factor: u32,
  pub throttle_a: f32,
  pub throttle_d: f32,
}

impl Default for PersonalityConfig {
  fn default() -> Self {
    Self {
      advertise: true,
      deauth: true,
      associate: true,
      channels: vec![], // Limits actions to these channels
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
