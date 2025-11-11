#![allow(clippy::cast_possible_truncation)]

use std::{collections::HashMap, mem, sync::Arc, time::Duration};

use anyhow::Result;
use parking_lot::{Mutex, RwLock};
use pwnagotchi_shared::{
  config::config_read,
  identity::Identity,
  mesh::peer::Peer,
  models::grid::{Advertisement, PeerResponse},
  sessions::manager::SessionManager,
  traits::{
    epoch::Epoch,
    general::{AdvertiserTrait, Component, CoreModule, CoreModules, Dependencies},
    grid::GridTrait,
    ui::ViewTrait,
  },
  utils::general::total_unique_handshakes,
};
use tokio::{sync::Mutex as AsyncMutex, task::JoinHandle, time::sleep};

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
  async fn init(&mut self, ctx: &CoreModules) -> Result<()> {
    self.advertiser = Some(Arc::new(AsyncMutex::new(AsyncAdvertiser::new(
      Arc::clone(&ctx.identity),
      Arc::clone(&ctx.epoch),
      Arc::clone(&ctx.view),
      Arc::clone(&ctx.grid),
      Arc::clone(&ctx.session_manager),
    ))));
    Ok(())
  }

  async fn start(&self) -> Result<Option<JoinHandle<()>>> {
    if let Some(ad) = &self.advertiser {
      let advertiser = Arc::clone(ad);
      advertiser.lock().await.start_advertising().await;
      let advertiser = Arc::clone(ad);
      let handle =
        tokio::spawn(async move { advertiser.lock().await.peer_and_advertisement_updater().await });
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
  pub grid: Arc<dyn GridTrait + Send + Sync>,
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
    if !config_read().personality.advertise {
      return;
    }
    let ad = Arc::clone(&self.advertisement);
    // Rounding errors my behated

    let serialized = {
      let ad = ad.lock();
      serde_json::to_value(&*ad).unwrap_or_default()
    };

    self.grid.set_advertisement_data(serialized);
    self.grid.advertise(Some(true));

    let grid = Arc::clone(&self.grid);

    self.view.on_state_change(
      "face",
      Box::new(move |_old: String, new: String| {
        on_face_change(Arc::clone(&grid), Arc::clone(&ad), new);
      }),
    );
  }

  async fn peer_and_advertisement_updater(&mut self) {
    sleep(Duration::from_secs(20)).await;
    loop {
      let peers = self.grid.peers().await.unwrap_or_default();
      self.merge_peers(peers);
      self.update_advertisement().await;
      sleep(Duration::from_secs(3)).await;
    }
  }
}

impl AsyncAdvertiser {
  pub fn new(
    identity: Arc<RwLock<Identity>>,
    epoch: Arc<RwLock<Epoch>>,
    view: Arc<dyn ViewTrait + Send + Sync>,
    grid: Arc<dyn GridTrait + Send + Sync>,
    sm: Arc<SessionManager>,
  ) -> Self {
    let epoch_data = Arc::clone(&epoch);

    let (epoch_num, epoch_handshakes) = {
      let e = epoch_data.read();
      (e.epoch, e.num_handshakes)
    };

    let config = config_read();

    let advertisement = Advertisement {
      name: config.main.name.to_string(),
      version: env!("CARGO_PKG_VERSION").to_string(),
      identity: identity.read().fingerprint().to_string(),
      face: config.faces.friend.to_string(),
      pwnd_run: epoch_handshakes,
      pwnd_total: 0,
      uptime: 0,
      epoch: epoch_num,
      policy: config.personality.clone(),
    };
    let adv_lock = Arc::new(Mutex::new(advertisement));

    Self {
      epoch,
      view,
      advertisement: adv_lock,
      sm,
      grid,
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
    let uptime = self.sm.get_session().read().started_at.elapsed();

    ad_mut.pwnd_run = session.read().state.handshakes.len() as u32;
    ad_mut.pwnd_total = total_unique_handshakes(&config_read().bettercap.handshakes) as u32;
    ad_mut.uptime = uptime.unwrap_or_default().as_secs() as u32;
    ad_mut.epoch = self.epoch.read().epoch;

    drop(ad_mut);

    let advertisement = Arc::clone(&self.advertisement);

    let ad = advertisement.lock();
    self.grid.set_advertisement_data(serde_json::to_value(&*ad).unwrap_or_default());
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

  fn merge_peers(&mut self, responses: Vec<PeerResponse>) {
    let mut previous_peers = mem::take(&mut self.peers);
    // Track peers seen in this poll to detect joins, updates, and departures.
    let mut updated_peers = HashMap::with_capacity(responses.len());
    let mut closest: Option<(String, i16)> = None;

    for response in responses {
      let Some(fingerprint) = response.fingerprint.clone() else {
        continue;
      };

      let mut peer = Peer::new(&response);

      if let Some(mut existing) = previous_peers.remove(&fingerprint) {
        existing.update(&peer);
        peer = existing;
      } else {
        self.on_new_peer(&peer);
      }

      if closest.as_ref().is_none_or(|(_, rssi)| peer.rssi > *rssi) {
        closest = Some((fingerprint.clone(), peer.rssi));
      }

      updated_peers.insert(fingerprint, peer);
    }

    for lost_peer in previous_peers.into_values() {
      self.on_lost_peer(&lost_peer);
    }

    self.closest_peer = closest.map(|(fingerprint, _)| fingerprint);
    self.peers = updated_peers;
  }
}

fn on_face_change(
  grid: Arc<dyn GridTrait + Send + Sync>,
  ad: Arc<Mutex<Advertisement>>,
  new: String,
) {
  let ad = Arc::clone(&ad);
  let mut ad_mut = ad.lock();
  ad_mut.face = new;
  drop(ad_mut);

  let advertisement = Arc::clone(&ad);
  grid.set_advertisement_data(serde_json::to_value(&*advertisement.lock()).unwrap_or_default());
}
