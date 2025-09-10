use std::{
  collections::{HashMap, HashSet},
  fs,
  sync::Arc,
  time::Duration,
};

use parking_lot::Mutex;
use pwnagotchi_shared::{
  config::config,
  logger::LOGGER,
  models::{
    agent::RunningMode,
    bettercap::BettercapSession,
    net::{AccessPoint, Station},
  },
  sessions::{manager::SessionManager, session::Session},
  traits::{automata::AgentObserver, ui::ViewTrait},
  types::{epoch::Activity, ui::StateValue},
};
use serde::{Deserialize, Serialize};

use crate::{
  ai::Epoch, bettercap::BettercapCommand, traits::bettercapcontroller::BettercapController,
};

pub struct Agent {
  pub observer: Arc<dyn AgentObserver + Send + Sync>,
  pub bettercap: Arc<dyn BettercapController>,
  pub epoch: Arc<Mutex<Epoch>>,
  pub view: Arc<dyn ViewTrait + Send + Sync>,

  pub mode: RunningMode,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
  pub event_type: String,
  pub data: HashMap<String, String>,
}

impl Agent {
  pub fn new(
    observer: &Arc<dyn AgentObserver + Send + Sync>,
    bettercap: &Arc<dyn BettercapController>,
    epoch: &Arc<Mutex<Epoch>>,
    view: &Arc<dyn ViewTrait + Send + Sync>,
  ) -> Self {
    Self::initialize();

    Self {
      observer: Arc::clone(observer),
      bettercap: Arc::clone(bettercap),
      epoch: Arc::clone(epoch),
      view: Arc::clone(view),
      mode: RunningMode::Manual,
    }
  }

  fn initialize() {
    let handshakes_path = &config().bettercap.handshakes.as_ref();
    if fs::metadata(handshakes_path).is_err()
      && let Err(e) = fs::create_dir_all(handshakes_path)
    {
      LOGGER.log_fatal("Agent", &format!("Failed to create handshakes dir: {e}"));
    }
  }

  pub fn start(&self) {
    self.observer.set_starting();
    self.observer.next_epoch();
    self.observer.set_ready();
  }

  /*fn reboot(&self) {
    LOGGER.log_info("Agent", "Rebooting agent...");
    self.observer.set_rebooting();
  }

  fn restart(&self, mode: Option<RunningMode>) {
    let mode = mode.unwrap_or(RunningMode::Auto);
  }*/

  pub async fn set_mode(&self, sm: &Arc<SessionManager>, mode: RunningMode) {
    let session = sm.get_session().await;

    sm.set_session(Session {
      started_at: session.started_at,
      supported_channels: session.supported_channels.clone(),
      mode,
      state: session.state.clone(),
    })
    .await;

    match mode {
      RunningMode::Auto => {
        self.view.set("mode", StateValue::Text("AUTO".into()));
      }
      RunningMode::Manual => {
        self.view.set("mode", StateValue::Text("MANU".into()));
      }
      RunningMode::Ai => {
        self.view.set("mode", StateValue::Text("AI".into()));
      }
      RunningMode::Custom => {
        self.view.set("mode", StateValue::Text("CUSTOM".into()));
      }
    }
  }

  pub async fn get_access_points_by_channel(
    &self,
    sm: &Arc<SessionManager>,
  ) -> Vec<(u8, Vec<AccessPoint>)> {
    let aps = self.get_access_points(sm).await;

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

  pub async fn recon(&self, sm: &Arc<SessionManager>) {
    let mut recon_time = config().personality.recon_time;
    let max_inactive = config().personality.max_inactive_scale;
    let recon_multiplier = config().personality.recon_inactive_multiplier;
    let channels = &config().personality.channels;

    LOGGER.log_debug("RECON", "Starting Recon");

    if self.epoch.lock().inactive_for >= max_inactive {
      recon_time *= recon_multiplier;
    }

    self.view.set("channel", StateValue::Text("*".into()));

    if channels.is_empty() {
      sm.get_session().await.state.write().current_channel = 0;
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

      let _ = self
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

  pub async fn associate(
    &self,
    sm: &Arc<SessionManager>,
    ap: &AccessPoint,
    mut throttle: Option<f32>,
  ) {
    if self.observer.is_stale() {
      LOGGER.log_debug("AGENT", &format!("Recon is stale, skipping association to {}", ap.mac));
      return;
    }

    if throttle.is_none() && config().personality.throttle_a.is_finite() {
      throttle = Some(config().personality.throttle_a);
    }

    if config().personality.associate && should_interact(&sm.get_session().await, &ap.mac) {
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
        .send(BettercapCommand::run(format!("wifi.assoc {mac}"), Some(tx)))
        .await;

      match rx.await {
        Ok(res) => match res {
          Ok(()) => {
            LOGGER.log_info(
              "AGENT",
              &format!("Associated with {} ({}) on channel {}", ap.mac, ap.hostname, ap.channel),
            );

            {
              self.epoch.lock().track(Activity::Association, Some(1));
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

  pub async fn deauth(
    &self,
    sm: &Arc<SessionManager>,
    ap: &AccessPoint,
    sta: &Station,
    mut throttle: Option<f32>,
  ) {
    if self.observer.is_stale() {
      LOGGER.log_debug("AGENT", &format!("Recon is stale, skipping deauth {}", sta.mac));
      return;
    }

    if throttle.is_none() && config().personality.throttle_d.is_finite() {
      throttle = Some(config().personality.throttle_d);
    }

    if config().personality.deauth && should_interact(&sm.get_session().await, &sta.mac) {
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

            self.epoch.lock().track(Activity::Deauth, Some(1));
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

  /*const fn restart(&mut self) {
    // TODO
  }

  const fn reboot(&mut self) {
    // TODO
  }*/

  pub fn set_access_points(&self, session: &Arc<Session>, aps: &Vec<AccessPoint>) {
    self.epoch.lock().observe(aps, &session.state.read().peers);
    session.state.write().access_points.clone_from(aps);
  }

  pub async fn get_access_points(&self, sm: &Arc<SessionManager>) -> Vec<AccessPoint> {
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

    self.set_access_points(&sm.get_session().await, &aps);

    aps
  }

  pub async fn set_channel(&self, sm: &Arc<SessionManager>, channel: u8) {
    if self.observer.is_stale() {
      LOGGER.log_debug("AGENT", &format!("Recon is stale, skipping channel switch to {channel}"));
      return;
    }

    LOGGER.log_debug("Agent", &format!("Attempting switch to Channel {channel}"));

    let mut wait = 0;
    let did_deauth = { self.epoch.lock().did_deauth };
    let did_associate = { self.epoch.lock().did_associate };
    if did_deauth {
      wait = config().personality.hop_recon_time;
    } else if did_associate {
      wait = config().personality.min_recon_time;
    }

    if channel != sm.get_session().await.state.read().current_channel {
      if sm.get_session().await.state.read().current_channel != 0 && wait > 0 {
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
          sm.get_session().await.state.write().current_channel = channel;
          self.epoch.lock().track(Activity::Hop, Some(1));
          self.view.set("channel", StateValue::Number(channel.into()));
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
  }
}

fn has_handshake(session: &Arc<Session>, bssid: &str) -> bool {
  session.state.read().handshakes.contains_key(bssid)
}

fn should_interact(session: &Arc<Session>, bssid: &str) -> bool {
  if has_handshake(session, bssid) {
    return false;
  } else if let std::collections::hash_map::Entry::Vacant(e) =
    session.state.write().history.entry(bssid.to_string())
  {
    e.insert(1);
    return true;
  }

  session.state.write().history.entry(bssid.to_string()).and_modify(|e| {
    *e += 1;
  });

  session.state.read().history[&bssid.to_string()] < config().personality.max_interactions
}

pub fn find_ap_sta_in_session(
  session: &Arc<Session>,
  sta_mac: &str,
  ap_mac: &str,
) -> Option<(AccessPoint, Station)> {
  let session = session.state.read();

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

pub async fn is_module_running(bc: &Arc<dyn BettercapController>, module: &str) -> bool {
  match bc.session().await {
    Ok(Some(session)) => session.modules.iter().any(|m| m.name == module && m.running),
    _ => false,
  }
}

pub async fn restart_module(bc: &Arc<dyn BettercapController>, module: &str) {
  let _ = bc.send(BettercapCommand::run(format!("{module} off; {module} on"), None)).await;
}

pub async fn stop_module(bc: &Arc<dyn BettercapController>, module: &str) {
  let _ = bc.send(BettercapCommand::run(format!("{module} off"), None)).await;
}

pub async fn start_module(bc: &Arc<dyn BettercapController>, module: &str) {
  let _ = bc.send(BettercapCommand::run(format!("{module} on"), None)).await;
}

pub fn get_total_aps(session: &Arc<Session>) -> usize {
  session.state.read().access_points.len()
}
