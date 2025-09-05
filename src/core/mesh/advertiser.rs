#![allow(clippy::cast_possible_truncation)]

use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::core::{
  ai::Epoch,
  config::{PersonalityConfig, config},
  grid::{advertise, set_advertisement_data},
  identity::Identity,
  log::LOGGER,
  sessions::manager::SessionManager,
  ui::{state::StateValue, view::View},
  utils::{self},
};

pub struct AsyncAdvertiser {
  pub epoch: Arc<Mutex<Epoch>>,
  pub advertisement: Advertisement,
  pub peers: HashMap<String, Advertisement>,
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
  pub fn new(epoch: Arc<Mutex<Epoch>>, identity: &Identity) -> Self {
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
      advertisement,
      peers: HashMap::new(),
      closest_peer: None,
    }
  }

  async fn update_advertisement(&mut self, sm: &Arc<SessionManager>) {
    self.advertisement.pwnd_run = sm.get_session().await.state.read().handshakes.len() as u32;
    self.advertisement.pwnd_total =
      utils::total_unique_handshakes(&config().main.handshakes_path) as u32;
    self.advertisement.uptime = 0;
    self.advertisement.epoch = self.epoch.lock().epoch as u32;
    set_advertisement_data(serde_json::to_value(self.advertisement.clone()).unwrap()).await;
  }

  pub async fn start_advertising(self, _sm: &Arc<SessionManager>, view: &Arc<View>) {
    if config().personality.advertise {
      set_advertisement_data(serde_json::to_value(self.advertisement.clone()).unwrap_or_default())
        .await;
      advertise(Some(true)).await;
      let advertiser = Arc::new(Mutex::new(self));
      let advertiser_clone = Arc::clone(&advertiser);
      view
        .on_state_change("face", move |old, new| advertiser_clone.lock().on_face_change(old, new));
    }
  }

  fn on_face_change(&mut self, _old: StateValue, new: StateValue) {
    let StateValue::Text(new) = new else {
      return;
    };

    if let Some(face_str) = Some(new) {
      self.advertisement.face = face_str;
    }

    tokio::spawn({
      let advertisement = self.advertisement.clone();
      async move {
        set_advertisement_data(serde_json::to_value(advertisement).unwrap_or_default()).await;
      }
    });
  }

  /*fn on_face_change(&self, _old: String, _new: String) {
      //self.advertisement.face = new.clone();
      // TODO: Update Grid
  }

  fn on_new_peer(&self, _peer: Peer) {
      //LOGGER.log_info("GRID", &format!("new peer {} detected ({} encounters)", peer.name, peer.encounters));
  }*/

  pub async fn advertisement_poller(&self) {
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;
    loop {
      LOGGER.log_debug("GRID", "Polling pwngrid-peer for peers...");

      // Do stuff

      tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
  }
}
