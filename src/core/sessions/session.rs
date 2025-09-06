use std::{borrow::Cow, collections::HashMap, sync::Arc, time::SystemTime};

use parking_lot::RwLock;

use crate::core::{
  agent::RunningMode,
  mesh::peer::Peer,
  models::net::{AccessPoint, Handshake},
};

#[derive(Debug, Clone)]
pub struct Session {
  pub started_at: SystemTime,
  pub supported_channels: Vec<u8>,
  pub mode: RunningMode,
  pub state: Arc<RwLock<SessionState>>,
}

#[derive(Debug, Clone)]
pub struct SessionState {
  pub current_channel: u8,
  pub total_aps: u32,
  pub aps_on_channel: u32,
  pub peers: Vec<Peer>,
  pub access_points: Vec<AccessPoint>,
  pub last_pwned: Option<Cow<'static, str>>,
  pub history: HashMap<String, u32>,
  pub handshakes: HashMap<String, Handshake>,
}

impl Default for Session {
  fn default() -> Self {
    Self::new()
  }
}

impl Session {
  pub fn new() -> Self {
    Self {
      started_at: std::time::SystemTime::now(),
      supported_channels: vec![],
      mode: RunningMode::Manual,
      state: Arc::new(RwLock::new(SessionState {
        current_channel: 0,
        total_aps: 0,
        aps_on_channel: 0,
        peers: vec![],
        access_points: vec![],
        last_pwned: None,
        history: HashMap::new(),
        handshakes: HashMap::new(),
      })),
    }
  }
}
