#![allow(clippy::cast_possible_truncation)]

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use parking_lot::{Mutex, RwLock};
use pwnagotchi_shared::{
  config::config,
  identity::Identity,
  logger::LOGGER,
  mesh::peer::Peer,
  models::grid::Advertisement,
  sessions::manager::SessionManager,
  traits::{
    epoch::Epoch,
    general::{AdvertiserTrait, Component, CoreModule, CoreModules, Dependencies},
    ui::ViewTrait,
  },
  utils::general::total_unique_handshakes,
};
use tokio::{sync::Mutex as AsyncMutex, task::JoinHandle};

use crate::grid::{advertise, peers, set_advertisement_data};

pub struct AdvertiserComponent {
  advertiser: Option<Arc<AsyncMutex<dyn AdvertiserTrait + Send + Sync>>>,
}

impl Dependencies for AdvertiserComponent {
  fn name(&self) -> &'static str {
    "AdvertiserComponent"
  }

  fn dependencies(&self) -> &[&str] {
    &[
      "Identity",
      "View",
      "Epoch",
      "SessionManager",
    ]
  }
}

#[async_trait::async_trait]
impl Component for AdvertiserComponent {
  async fn init(&mut self, _ctx: &CoreModules) -> Result<()> {
    Ok(())
  }

  async fn start(&self) -> Result<Option<JoinHandle<()>>> {
    if let Some(ad) = &self.advertiser {
      let advertiser = Arc::clone(ad);
      advertiser.lock().await.start_advertising().await;
      let advertiser = Arc::clone(ad);
      let handle = tokio::spawn(async move { advertiser.lock().await.peer_poller().await });
      return Ok(Some(handle));
    }
    Ok(None)
  }
}

impl Default for AdvertiserComponent {
  fn default() -> Self {
    Self::new()
  }
}

impl AdvertiserComponent {
  pub const fn new() -> Self {
    Self { advertiser: None }
  }
}

pub struct AsyncAdvertiser {
  pub epoch: Arc<RwLock<Epoch>>,
  pub advertisement: Arc<Mutex<Advertisement>>,
  pub sm: Arc<SessionManager>,
  pub view: Arc<dyn ViewTrait + Send + Sync>,
  pub peers: HashMap<String, Peer>,
  pub closest_peer: Option<String>,
}

impl CoreModule for AsyncAdvertiser {
  fn name(&self) -> &'static str {
    "AsyncAdvertiser"
  }

  fn dependencies(&self) -> &[&'static str] {
    &[
      "Identity",
      "View",
      "Epoch",
      "SessionManager",
    ]
  }
}

#[async_trait::async_trait]
impl AdvertiserTrait for AsyncAdvertiser {
  async fn start_advertising(&self) {
    if !config().personality.advertise {
      return;
    }
    let ad = Arc::clone(&self.advertisement);
    set_advertisement_data(serde_json::to_value(&*ad.lock()).unwrap_or_default());
    advertise(Some(true));

    let clone = Arc::clone(&ad);
    self.view.on_state_change(
      "face",
      Box::new(move |old: String, new: String| {
        on_face_change(Arc::clone(&clone), old.clone(), new.clone())
      }),
    );
  }

  async fn peer_poller(&mut self) {
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
          let is_closer = if let Some(ref closest) = self.closest_peer {
            peer.rssi > self.peers.get(closest).map_or(-100, |p| p.rssi)
          } else {
            true
          };
          if is_closer {
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

impl AsyncAdvertiser {
  pub fn new(
    identity: Arc<RwLock<Identity>>,
    epoch: Arc<RwLock<Epoch>>,
    view: Arc<dyn ViewTrait + Send + Sync>,
    sm: Arc<SessionManager>,
  ) -> Self {
    let epoch_data = Arc::clone(&epoch);

    let (epoch_num, epoch_handshakes) = {
      let e = epoch_data.read();
      (e.epoch as u32, e.num_handshakes)
    };

    let advertisement = Advertisement {
      name: config().main.name.to_string(),
      version: env!("CARGO_PKG_VERSION").to_string(),
      identity: identity.read().fingerprint().to_string(),
      face: config().faces.friend.to_string(),
      pwnd_run: epoch_handshakes,
      pwnd_total: 0,
      uptime: 0,
      epoch: epoch_num,
      policy: config().personality.clone(),
    };
    let adv_lock = Arc::new(Mutex::new(advertisement));

    Self {
      epoch,
      view,
      advertisement: adv_lock,
      sm,
      peers: HashMap::new(),
      closest_peer: None,
    }
  }

  pub fn fingerprint(&self) -> String {
    self.advertisement.lock().identity.clone()
  }

  #[allow(dead_code)]
  async fn update_advertisement(&mut self) {
    let session = self.sm.get_session();
    let ad_arc = Arc::clone(&self.advertisement);
    let mut ad_mut = ad_arc.lock();
    ad_mut.pwnd_run = session.read().state.handshakes.len() as u32;
    ad_mut.pwnd_total = total_unique_handshakes(&config().bettercap.handshakes) as u32;
    ad_mut.uptime = 0;
    ad_mut.epoch = self.epoch.read().epoch as u32;

    drop(ad_mut);

    let advertisement = Arc::clone(&self.advertisement);

    let ad = advertisement.lock();
    set_advertisement_data(serde_json::to_value(&*ad).unwrap_or_default());
  }

  pub fn cumulative_encounters(&self) -> u32 {
    self.peers.values().map(|p| p.encounters).sum()
  }

  fn on_new_peer(&self, peer: &Peer) {
    self.view.on_new_peer(peer);
  }

  fn on_lost_peer(&self, peer: &Peer) {
    self.view.on_lost_peer(peer);
  }
}

fn on_face_change(ad: Arc<Mutex<Advertisement>>, _old: String, new: String) {
  let ad = Arc::clone(&ad);
  let mut ad_mut = ad.lock();
  ad_mut.face = new.clone();
  drop(ad_mut);

  let advertisement = Arc::clone(&ad);
  set_advertisement_data(serde_json::to_value(&*advertisement.lock()).unwrap_or_default());
}
