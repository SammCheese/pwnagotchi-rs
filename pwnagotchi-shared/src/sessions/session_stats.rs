use std::{collections::HashMap, time::SystemTime};

use crate::{mesh::peer::Peer, utils::general::format_duration_human};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EpochStats {
  pub epochs: usize,
  pub train_epochs: usize,
  pub min_reward: f64,
  pub max_reward: f64,
  pub avg_reward: f64,
}

impl Default for EpochStats {
  fn default() -> Self {
    Self {
      epochs: 0,
      train_epochs: 0,
      min_reward: f64::MAX,
      max_reward: f64::MIN,
      avg_reward: 0.0,
    }
  }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PeerStats {
  pub peers: usize,
  pub last_peer: Option<Peer>,
  pub history: HashMap<String, u32>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SessionStats {
  pub id: String,
  pub start: Option<SystemTime>,
  pub stop: Option<SystemTime>,
  pub duration_secs: Option<u64>,
  pub human_duration: Option<String>,

  pub deauthed: usize,
  pub associated: usize,
  pub handshakes: usize,

  pub epochs: EpochStats,
  pub peers: PeerStats,
}

impl SessionStats {
  pub fn finalize(&mut self) {
    if let (Some(start), Some(stop)) = (self.start, self.stop)
      && let Ok(duration) = stop.duration_since(start)
    {
      self.duration_secs = Some(duration.as_secs());
      self.human_duration = Some(format_duration_human(duration));
    }
    if self.epochs.epochs > 0 {
      self.epochs.avg_reward /= self.epochs.epochs as f64;
    }
  }
}
