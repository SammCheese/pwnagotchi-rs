#![allow(clippy::cast_possible_truncation)]
use std::{collections::HashMap};

use crate::core::{agent::Agent, config::{config, PersonalityConfig}, identity::Identity, utils};


pub struct AsyncAdvertiser {
  pub agent: Agent,
  pub identity: Identity,
  pub advertisement: Advertisement,
  pub peers: HashMap<String, Advertisement>,
  pub closest_peer: Option<String>,
}

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

impl AsyncAdvertiser {
  pub fn new(agent: Agent, identity: Identity) -> Self {
    let advertisement = Advertisement {
      name: config().main.name.clone(),
      version: env!("CARGO_PKG_VERSION").to_string(),
      identity: identity.fingerprint(),
      face: "(._.)".to_string(),
      pwnd_run: 0,
      pwnd_total: 0,
      uptime: 0,
      epoch: 0,
      policy: config().personality.clone(),
    };

    Self {
      agent,
      identity,
      advertisement,
      peers: HashMap::new(),
      closest_peer: None,
    }
  }

  fn update_advertisement(&mut self) {
    self.advertisement.pwnd_run = self.agent.handshakes.len() as u32;
    self.advertisement.pwnd_total = utils::total_unique_handshakes(&config().main.handshakes_path) as u32;
    self.advertisement.uptime = 0;
    self.advertisement.epoch = self.agent.automata.epoch.epoch as u32;
  }

  pub fn start_advertising(self) {
    if config().personality.advertise {
      // TODO
    }
  }
}