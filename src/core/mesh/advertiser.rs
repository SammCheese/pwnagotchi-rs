#![allow(clippy::cast_possible_truncation)]

use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::core::{
  ai::Epoch,
  config::{PersonalityConfig, config},
  grid::{advertise, peers, set_advertisement_data},
  identity::Identity,
  log::LOGGER,
  mesh::peer::Peer,
  sessions::manager::SessionManager,
  ui::{state::StateValue, view::View},
  utils::{self},
};

pub struct AsyncAdvertiser {
  pub epoch: Arc<Mutex<Epoch>>,
  pub advertisement: Advertisement,
  pub view: Option<Arc<View>>,
  pub peers: HashMap<String, Peer>,
  pub closest_peer: Option<String>,
}

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

impl AsyncAdvertiser {
  pub fn new(epoch: Arc<Mutex<Epoch>>, identity: &Identity, view: Option<Arc<View>>) -> Self {
    let advertisement = Advertisement {
      name: config().main.name.to_string(),
      version: env!("CARGO_PKG_VERSION").to_string(),
      identity: identity.fingerprint().to_string(),
      face: config().faces.friend.to_string(),
      pwnd_run: 0,
      pwnd_total: 0,
      uptime: 0,
      epoch: 0,
      policy: config().personality.clone(),
    };

    Self {
      epoch,
      view,
      advertisement,
      peers: HashMap::new(),
      closest_peer: None,
    }
  }

  pub fn fingerprint(&self) -> &str {
    &self.advertisement.identity
  }

  async fn update_advertisement(&mut self, sm: &Arc<SessionManager>) {
    self.advertisement.pwnd_run = sm.get_session().await.state.read().handshakes.len() as u32;
    self.advertisement.pwnd_total =
      utils::total_unique_handshakes(&config().main.handshakes_path) as u32;
    self.advertisement.uptime = 0;
    self.advertisement.epoch = self.epoch.lock().epoch as u32;
    set_advertisement_data(serde_json::to_value(self.advertisement.clone()).unwrap()).await;
  }

  fn on_face_change(&mut self, _old: StateValue, new: StateValue) {
    let StateValue::Text(new) = new else {
      return;
    };

    if let Some(face_str) = Some(new) {
      self.advertisement.face = face_str;
    }

    tokio::task::spawn({
      let advertisement = self.advertisement.clone();
      async move {
        set_advertisement_data(serde_json::to_value(advertisement).unwrap_or_default()).await;
      }
    });
  }

  pub fn cumulative_encounters(&self) -> u32 {
    self.peers.values().map(|p| p.encounters).sum()
  }

  fn on_new_peer(&self, peer: &Peer) {
    if let Some(view) = &self.view {
      view.on_new_peer(peer);
    }
  }

  fn on_lost_peer(&self, peer: &Peer) {
    if let Some(view) = &self.view {
      view.on_lost_peer(peer);
    }
  }

  pub async fn advertisement_poller(&mut self) {
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;
    loop {
      LOGGER.log_debug("GRID", "Polling pwngrid-peer for peers...");

      let peers_opt = peers().await;
      let mut new_peers = HashMap::new();
      self.closest_peer = None;

      if let Some(peers_vec) = peers_opt {
        for p in &peers_vec {
          let peer = Peer::new(p);
          new_peers.insert(p.fingerprint.clone().unwrap_or_default(), peer.clone());
          if self.closest_peer.is_none()
            || peer.rssi
              > self.peers.get(self.closest_peer.as_ref().unwrap()).map_or(-100, |p| p.rssi)
          {
            self.closest_peer = Some(p.fingerprint.clone().unwrap_or_default());
          }
        }
      }

      let to_delete = self
        .peers
        .keys()
        .filter(|k| !new_peers.contains_key(*k))
        .cloned()
        .collect::<Vec<_>>();

      for k in to_delete {
        if let Some(p) = self.peers.remove(&k) {
          self.on_lost_peer(&p);
        }
      }

      for (k, v) in &new_peers {
        if !self.peers.contains_key(k) {
          self.on_new_peer(v);
        } else if let Some(existing_peer) = self.peers.get_mut(k) {
          existing_peer.update(v);
        }
      }

      tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
  }
}

pub async fn start_advertising(
  advertiser: &Arc<Mutex<AsyncAdvertiser>>,
  _sm: &Arc<SessionManager>,
  view: &Arc<View>,
) {
  if config().personality.advertise {
    let value = advertiser.lock().advertisement.clone();
    set_advertisement_data(serde_json::to_value(value).unwrap_or_default()).await;
    advertise(Some(true)).await;
    let advertiser_clone = Arc::clone(advertiser);
    view.on_state_change("face", move |old, new| advertiser_clone.lock().on_face_change(old, new));
  }
}
