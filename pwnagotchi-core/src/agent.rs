use std::{
  collections::{HashMap, HashSet},
  fs,
  sync::Arc,
  time::Duration,
};

use anyhow::Result;
use parking_lot::RwLock;
use pwnagotchi_macros::hookable;
use pwnagotchi_shared::{
  config::config,
  logger::LOGGER,
  models::{
    agent::RunningMode,
    bettercap::BettercapSession,
    net::{AccessPoint, Station},
  },
  sessions::{manager::SessionManager, session::Session},
  traits::{
    agent::AgentTrait,
    automata::AutomataTrait,
    bettercap::{BettercapCommand, BettercapTrait},
    epoch::Epoch,
    general::{Component, CoreModule, CoreModules, Dependencies},
    ui::ViewTrait,
  },
  types::epoch::Activity,
};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

pub struct AgentComponent {
  agent: Option<Arc<dyn AgentTrait + Send + Sync>>,
}

impl Default for AgentComponent {
  fn default() -> Self {
    Self::new()
  }
}

impl AgentComponent {
  pub fn new() -> Self {
    Self { agent: None }
  }
}

impl Dependencies for AgentComponent {
  fn name(&self) -> &'static str {
    "AgentComponent"
  }

  fn dependencies(&self) -> &[&str] {
    &[
      "Identity",
      "Bettercap",
      "Automata",
      "Epoch",
      "View",
      "SessionManager",
    ]
  }
}

#[async_trait::async_trait]
impl Component for AgentComponent {
  async fn init(&mut self, _ctx: &CoreModules) -> Result<()> {
    let handshakes_path = &config().bettercap.handshakes.as_ref();
    if fs::metadata(handshakes_path).is_err()
      && let Err(e) = fs::create_dir_all(handshakes_path)
    {
      LOGGER.log_fatal("Agent", &format!("Failed to create handshakes dir: {e}"));
    }

    Ok(())
  }

  async fn start(&self) -> Result<Option<JoinHandle<()>>> {
    if let Some(agent) = &self.agent {
      agent.start_pwnagotchi();
    }
    Ok(None)
  }
}

pub struct Agent {
  pub sm: Arc<SessionManager>,
  pub observer: Arc<dyn AutomataTrait + Send + Sync>,
  pub bettercap: Arc<dyn BettercapTrait + Send + Sync>,
  pub epoch: Arc<RwLock<Epoch>>,
  pub view: Arc<dyn ViewTrait + Send + Sync>,
  pub mode: RunningMode,
}

impl CoreModule for Agent {
  fn name(&self) -> &'static str {
    "Agent"
  }

  fn dependencies(&self) -> &[&'static str] {
    &[
      "Identity",
      "Bettercap",
      "Automata",
      "Epoch",
      "View",
      "SessionManager",
    ]
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
  pub event_type: String,
  pub data: HashMap<String, String>,
}

#[async_trait::async_trait]
impl AgentTrait for Agent {
  async fn set_mode(&self, mode: RunningMode) {
    self.set_mode(mode).await;
  }

  async fn recon(&self) {
    self.recon().await;
  }

  async fn associate(&self, ap: &AccessPoint, throttle: Option<f32>) {
    self.associate(ap, throttle).await;
  }

  async fn deauth(&self, ap: &AccessPoint, sta: &Station, throttle: Option<f32>) {
    self.deauth(ap, sta, throttle).await;
  }

  async fn set_channel(&self, channel: u8) {
    self.set_channel(channel).await;
  }

  async fn get_access_points_by_channel(&self) -> Vec<(u8, Vec<AccessPoint>)> {
    self.get_access_points_by_channel().await
  }

  fn start_pwnagotchi(&self) {
    self.start_pwnagotchi();
  }

  fn reboot(&self) {
    self.reboot();
  }

  fn restart(&self, mode: Option<RunningMode>) {
    self.restart(mode);
  }
}

#[hookable]
impl Agent {
  pub fn new(
    observer: Arc<dyn AutomataTrait + Send + Sync>,
    bettercap: Arc<dyn BettercapTrait + Send + Sync>,
    epoch: Arc<RwLock<Epoch>>,
    view: Arc<dyn ViewTrait + Send + Sync>,
    sm: Arc<SessionManager>,
  ) -> Self {
    Self {
      observer,
      bettercap,
      epoch,
      view,
      sm,
      mode: RunningMode::Manual,
    }
  }

  #[hookable]
  pub async fn set_access_points(&self, aps: &Vec<AccessPoint>) {
    let session = self.sm.get_session();
    self.epoch.write().observe(aps, &session.read().state.peers);
    session.write().state.access_points.clone_from(aps);
  }

  #[hookable]
  pub async fn get_access_points(&self) -> Vec<AccessPoint> {
    let ignored: HashSet<String> =
      config().main.whitelist.iter().map(|s| s.to_lowercase()).collect();
    let mut aps: Vec<AccessPoint> = Vec::new();

    if let Ok(Some(session)) = self.bettercap.session().await {
      for ap in session.wifi.aps {
        LOGGER.log_debug("Agent", &format!("Got host {}", ap.hostname));

        if ap.encryption.is_empty() || ap.encryption.eq_ignore_ascii_case("OPEN") {
          continue;
        }

        let mac = ap.mac.to_lowercase();
        let ssid = ap.hostname.to_lowercase();

        if ignored.contains(&mac) || ignored.contains(&ssid) {
          continue;
        }

        aps.push(ap);
      }
    }

    aps.sort_by_key(|ap| ap.channel);

    self.set_access_points(&aps).await;

    aps
  }

  #[hookable]
  fn should_interact(&self, bssid: &str) -> bool {
    let session = self.sm.get_session();
    if has_handshake(&session, bssid) {
      return false;
    } else if let std::collections::hash_map::Entry::Vacant(e) =
      session.write().state.history.entry(bssid.to_string())
    {
      e.insert(1);
      return true;
    }

    session.write().state.history.entry(bssid.to_string()).and_modify(|e| {
      *e += 1;
    });

    session.read().state.history[&bssid.to_string()] < config().personality.max_interactions
  }

  #[hookable]
  async fn set_mode(&self, mode: RunningMode) {
    let session = self.sm.get_session();
    let (started_at, supported_channels, state) = {
      let s = session.read();
      (s.started_at, s.supported_channels.clone(), s.state.clone())
    };

    self.sm.set_session(Session {
      started_at,
      supported_channels,
      mode,
      state,
    });

    drop(session);

    match mode {
      RunningMode::Auto => {
        self.view.set("mode", "AUTO".into());
      }
      RunningMode::Manual => {
        self.view.set("mode", "MANUAL".into());
      }
      RunningMode::Ai => {
        self.view.set("mode", "AI".into());
      }
      RunningMode::Custom => {
        self.view.set("mode", "CUSTOM".into());
      }
    }
  }

  #[hookable]
  async fn recon(&self) {
    let mut recon_time = config().personality.recon_time;
    let max_inactive = config().personality.max_inactive_scale;
    let recon_multiplier = config().personality.recon_inactive_multiplier;
    let channels = &config().personality.channels;

    LOGGER.log_debug("RECON", "Starting Recon");

    if self.epoch.read().inactive_for >= max_inactive {
      recon_time *= recon_multiplier;
    }

    self.view.set("channel", "*".into());

    if channels.is_empty() {
      self.sm.get_session().write().state.current_channel = 0;

      LOGGER.log_info("RECON", "Listening on all available channels.");
      let (tx, rx) = tokio::sync::oneshot::channel();
      let _ = self
        .bettercap
        .send(BettercapCommand::run("wifi.recon.channel clear", Some(tx)))
        .await;

      if let Err(_e) = rx.await {
        LOGGER.log_error("RECON", "Failed to set channels");
      }
    } else {
      let channel_str = channels
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<HashSet<_>>() // deduplicate
        .into_iter()
        .collect::<Vec<_>>()
        .join(",");

      let (tx, rx) = tokio::sync::oneshot::channel();

      let _ = &self
        .bettercap
        .send(BettercapCommand::run(format!("wifi.recon.channel {channel_str}"), Some(tx)))
        .await;

      if let Err(e) = rx.await {
        LOGGER.log_error("RECON", &format!("Failed to set recon channel: {e}"));
      }
    }

    LOGGER.log_info("RECON", &format!("Recon time set to {recon_time} seconds"));
    self.observer.wait_for(recon_time, Some(false)).await;
  }

  #[hookable]
  #[allow(unused_mut)]
  async fn associate(&self, ap: &AccessPoint, mut throttle: Option<f32>) {
    if self.observer.is_stale() {
      LOGGER.log_debug("AGENT", &format!("Recon is stale, skipping association to {}", ap.mac));
      return;
    }

    if throttle.is_none() && config().personality.throttle_a.is_finite() {
      throttle = Some(config().personality.throttle_a);
    }

    if config().personality.associate && self.should_interact(&ap.mac) {
      self.view.on_assoc(ap);

      LOGGER.log_info(
        "AGENT",
        &format!(
          "sending association frame to {} ({}) on channel {} ({} clients), {} dBm",
          ap.mac,
          ap.hostname,
          ap.channel,
          ap.clients.len(),
          ap.rssi
        ),
      );

      let mac = &ap.mac;

      let (tx, rx) = tokio::sync::oneshot::channel();

      let _ = self
        .bettercap
        .send(BettercapCommand::run(format!("wifi.associate {mac}"), Some(tx)))
        .await;

      match rx.await {
        Ok(res) => match res {
          Ok(()) => {
            LOGGER.log_info(
              "AGENT",
              &format!("Associated with {} ({}) on channel {}", ap.mac, ap.hostname, ap.channel),
            );

            {
              self.epoch.write().track(Activity::Association, Some(1));
            }
          }
          Err(e) => {
            self.observer.on_error(ap, e.to_string().as_str());
          }
        },
        Err(e) => {
          self.observer.on_error(ap, e.to_string().as_str());
        }
      }

      if let Some(throttle) = throttle {
        LOGGER.log_debug("AGENT", &format!("Throttling association for {throttle} seconds"));
        tokio::time::sleep(Duration::from_secs_f32(throttle)).await;
      }

      self.view.on_normal();
    }
  }

  #[hookable]
  #[allow(unused_mut)]
  async fn deauth(&self, ap: &AccessPoint, sta: &Station, mut throttle: Option<f32>) {
    if self.observer.is_stale() {
      LOGGER.log_debug("AGENT", &format!("Recon is stale, skipping deauth {}", sta.mac));
      return;
    }

    if throttle.is_none() && config().personality.throttle_d.is_finite() {
      throttle = Some(config().personality.throttle_d);
    }

    if config().personality.deauth && self.should_interact(&sta.mac) {
      self.view.on_deauth(sta);

      LOGGER.log_info(
        "AGENT",
        &format!(
          "deauthing {} ({}) on channel {} ({} clients), {} dBm",
          ap.mac,
          ap.hostname,
          ap.channel,
          ap.clients.len(),
          ap.rssi
        ),
      );

      let mac = &sta.mac;
      let (tx, rx) = tokio::sync::oneshot::channel();

      let _ = self
        .bettercap
        .send(BettercapCommand::run(format!("wifi.deauth {mac}"), Some(tx)))
        .await;

      match rx.await {
        Ok(res) => match res {
          Ok(()) => {
            LOGGER.log_info(
              "AGENT",
              &format!(
                "Deauthenticated {} from {} on channel {}",
                sta.mac, ap.hostname, ap.channel
              ),
            );

            self.epoch.write().track(Activity::Deauth, Some(1));
          }
          Err(e) => {
            self.observer.on_error(ap, e.to_string().as_str());
          }
        },
        Err(e) => {
          self.observer.on_error(ap, e.to_string().as_str());
        }
      }

      if let Some(throttle) = throttle {
        LOGGER.log_debug("AGENT", &format!("Throttling deauth for {throttle} seconds"));
        tokio::time::sleep(Duration::from_secs_f32(throttle)).await;
      }

      self.view.on_normal();
    }
  }

  #[hookable]
  async fn set_channel(&self, channel: u8) {
    if self.observer.is_stale() {
      LOGGER.log_debug("AGENT", &format!("Recon is stale, skipping channel switch to {channel}"));
      return;
    }

    LOGGER.log_debug("Agent", &format!("Attempting switch to Channel {channel}"));

    let mut wait = 0;
    let did_deauth = self.epoch.read().did_deauth;
    let did_associate = self.epoch.read().did_associate;
    if did_deauth {
      wait = config().personality.hop_recon_time;
    } else if did_associate {
      wait = config().personality.min_recon_time;
    }

    let session = self.sm.get_session();
    if session.read().state.current_channel == channel {
      return;
    }

    if self.sm.get_session().read().state.current_channel != 0 && wait > 0 {
      LOGGER.log_debug("AGENT", &format!("Waiting {wait} seconds before switching channel"));
      self.observer.wait_for(wait, None).await;
    }

    let chs = channel.to_string();
    let (tx, rx) = tokio::sync::oneshot::channel();

    let _ = self
      .bettercap
      .send(BettercapCommand::run(format!("wifi.recon.channel {chs}"), Some(tx)))
      .await;

    match rx.await {
      Ok(Ok(())) => {
        self.sm.get_session().write().state.current_channel = channel;
        self.epoch.write().track(Activity::Hop, Some(1));
        self.view.set("channel", channel.to_string());
        LOGGER.log_info("AGENT", &format!("Switched to channel {channel}"));
      }
      Ok(Err(e)) => {
        LOGGER.log_error("AGENT", &format!("Failed to switch channel: {e}"));
      }
      Err(e) => {
        LOGGER.log_error("AGENT", &format!("Failed to receive response: {e}"));
      }
    }
  }

  #[hookable]
  async fn get_access_points_by_channel(&self) -> Vec<(u8, Vec<AccessPoint>)> {
    let aps = self.get_access_points().await;

    let channels: HashSet<u8> = config().personality.channels.iter().copied().collect();
    let mut grouped: HashMap<u8, Vec<AccessPoint>> = HashMap::new();

    LOGGER.log_debug("Agent", &format!("{} APS", aps.len()));

    for ap in aps {
      if channels.is_empty() || channels.contains(&ap.channel) {
        grouped.entry(ap.channel).or_default().push(ap);
      }
    }

    LOGGER.log_debug("Agent", &format!("Found {} populated channels", grouped.len()));

    let mut grouped_vec: Vec<(u8, Vec<AccessPoint>)> = grouped.into_iter().collect();

    grouped_vec.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));

    grouped_vec
  }

  #[hookable]
  fn start_pwnagotchi(&self) {
    self.observer.set_starting();
    self.observer.next_epoch();
    self.observer.set_ready();
  }

  #[hookable]
  fn reboot(&self) {
    LOGGER.log_info("Agent", "Rebooting agent...");
    self.observer.set_rebooting();
  }

  #[hookable]
  fn restart(&self, mode: Option<RunningMode>) {
    let _mode = mode.unwrap_or(RunningMode::Auto);
  }
}

fn has_handshake(session: &RwLock<Session>, bssid: &str) -> bool {
  session.read().state.handshakes.contains_key(bssid)
}

pub fn find_ap_sta_in_session(
  session: &Arc<RwLock<Session>>,
  sta_mac: &str,
  ap_mac: &str,
) -> Option<(AccessPoint, Station)> {
  let session = &session.read().state;

  session.access_points.iter().find(|ap| ap.mac == ap_mac).and_then(|ap| {
    ap.clients
      .iter()
      .find(|sta| sta.mac == sta_mac)
      .map(|sta| (ap.clone(), sta.clone()))
  })
}

pub fn find_ap_sta_in<'a>(
  sta_mac: &str,
  ap_mac: &str,
  session: Option<&'a BettercapSession>,
) -> Option<(&'a AccessPoint, &'a Station)> {
  session.and_then(|session| {
    session
      .wifi
      .aps
      .iter()
      .find(|ap| ap.mac == ap_mac)
      .and_then(|ap| ap.clients.iter().find(|sta| sta.mac == sta_mac).map(|sta| (ap, sta)))
  })
}

pub async fn is_module_running(bc: &Arc<dyn BettercapTrait + Send + Sync>, module: &str) -> bool {
  match bc.session().await {
    Ok(Some(session)) => session.modules.iter().any(|m| m.name == module && m.running),
    _ => false,
  }
}

pub async fn restart_module(bc: &Arc<dyn BettercapTrait + Send + Sync>, module: &str) {
  let _ = bc.run_fire_and_forget(format!("{module} off; {module} on")).await;
}

pub async fn stop_module(bc: &Arc<dyn BettercapTrait + Send + Sync>, module: &str) {
  let _ = bc.run_fire_and_forget(format!("{module} off")).await;
  LOGGER.log_info("Agent", &format!("Stopped module: {module}"));
}

pub async fn start_module(bc: &Arc<dyn BettercapTrait + Send + Sync>, module: &str) {
  let _ = bc.run_fire_and_forget(format!("{module} on")).await;
  LOGGER.log_info("Agent", &format!("started module: {module}"));
}

pub fn get_total_aps(session: &RwLock<Session>) -> usize {
  session.read().state.access_points.len()
}
