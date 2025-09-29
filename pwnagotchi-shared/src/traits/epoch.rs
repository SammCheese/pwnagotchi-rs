use std::time::Instant;

use tokio::sync::mpsc::{Receiver, Sender};

pub struct Epoch {
  pub obs_tx: Sender<Observation>,
  pub obs_rx: Receiver<Observation>,
  pub data_tx: Sender<EpochData>,
  pub data_rx: Receiver<EpochData>,
  pub epoch: u64,

  pub inactive_for: u32,
  pub active_for: u32,
  pub blind_for: u32,
  pub sad_for: u32,
  pub bored_for: u32,

  pub did_deauth: bool,
  pub num_deauths: u32,
  pub did_associate: bool,
  pub num_assocs: u32,
  pub num_missed: u32,
  pub did_handshakes: bool,
  pub num_handshakes: u32,
  pub num_hops: u32,
  pub num_slept: u32,
  pub num_peers: u32,
  pub total_bond_factor: f32,
  pub avg_bond_factor: f32,
  pub any_activity: bool,

  pub epoch_start: Instant,
  pub epoch_duration: f64,

  pub non_overlapping_channels: Vec<String>,
  pub observation: Observation,
  pub observation_ready: bool,
  pub epoch_data: EpochData,
  pub epoch_data_ready: bool,
}

#[derive(Clone, Default)]
pub struct EpochData {
  pub duration_secs: f64,
  pub slept_for_secs: f64,
  pub blind_for_epochs: u32,
  pub inactive_for_epochs: u32,
  pub active_for_epochs: u32,
  pub sad_for_epochs: u32,
  pub bored_for_epochs: u32,
  pub missed_interactions: u32,
  pub num_hops: u32,
  pub num_peers: u32,
  pub tot_bond: f32,
  pub avg_bond: f32,
  pub num_deauths: u32,
  pub num_associations: u32,
  pub num_handshakes: u32,
  pub cpu_load: f32,
  pub mem_usage: f32,
  pub temperature: f32,
  pub reward: f64,
}

#[derive(Clone)]
pub struct Observation {
  pub aps: Vec<f32>,
  pub sta: Vec<f32>,
  pub peers: Vec<f32>,
}
