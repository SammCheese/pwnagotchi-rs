use std::{fmt::Write, sync::Arc};

use anyhow::Result;
use parking_lot::RwLock;
use pwnagotchi_shared::{
  config::config,
  logger::LOGGER,
  models::net::Handshake,
  sessions::manager::SessionManager,
  traits::{
    bettercap::{BettercapCommand, BettercapTrait},
    epoch::Epoch,
    general::{Component, CoreModules, Dependencies},
    ui::ViewTrait,
  },
  types::epoch::Activity,
  utils::general::{hostname_or_mac, total_unique_handshakes},
};
use serde_json::Value;
use tokio::{sync::mpsc, task::JoinHandle};

use crate::agent::find_ap_sta_in_session;

pub struct EventListenerComponent {
  eventlistener: Option<Arc<EventListener>>,
}

impl Default for EventListenerComponent {
  fn default() -> Self {
    Self::new()
  }
}

impl EventListenerComponent {
  pub fn new() -> Self {
    Self { eventlistener: None }
  }
}

impl Dependencies for EventListenerComponent {
  fn name(&self) -> &'static str {
    "EventListenerComponent"
  }

  fn dependencies(&self) -> &[&str] {
    &[
      "Bettercap",
      "SessionManager",
      "Epoch",
      "View",
    ]
  }
}

#[async_trait::async_trait]
impl Component for EventListenerComponent {
  async fn init(&mut self, ctx: &CoreModules) -> Result<()> {
    let (bc, sm, epoch, view) = (&ctx.bettercap, &ctx.session_manager, &ctx.epoch, &ctx.view);
    let sm = Arc::clone(sm);
    let epoch = Arc::clone(epoch);
    let bettercap = Arc::clone(bc);
    let view = Arc::clone(view);
    let eventlistener = EventListener::new(sm, bettercap, epoch, view);
    self.eventlistener = Some(Arc::new(eventlistener));

    Ok(())
  }

  async fn start(&self) -> Result<Option<JoinHandle<()>>> {
    if let Some(ev) = &self.eventlistener {
      LOGGER.log_info("Agent", "Starting EventListener component");
      let ev = Arc::clone(ev);
      let handle = tokio::spawn(async move {
        start_event_loop(&ev.sm, &ev.bettercap, &ev.epoch, &ev.view).await;
      });
      return Ok(Some(handle));
    }
    Ok(None)
  }
}

pub struct EventListener {
  sm: Arc<SessionManager>,
  bettercap: Arc<dyn BettercapTrait + Send + Sync>,
  epoch: Arc<RwLock<Epoch>>,
  view: Arc<dyn ViewTrait + Send + Sync>,
}

impl EventListener {
  pub fn new(
    sm: Arc<SessionManager>,
    bettercap: Arc<dyn BettercapTrait + Send + Sync>,
    epoch: Arc<RwLock<Epoch>>,
    view: Arc<dyn ViewTrait + Send + Sync>,
  ) -> Self {
    Self { sm, bettercap, epoch, view }
  }
}

pub async fn start_event_loop(
  sm: &Arc<SessionManager>,
  bc: &Arc<dyn BettercapTrait + Send + Sync>,
  epoch: &Arc<RwLock<Epoch>>,
  view: &Arc<dyn ViewTrait + Send + Sync>,
) {
  LOGGER.log_info("Agent", "Starting event loop");
  let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(1000);

  tokio::spawn({
    let tx = tx;
    let bc = Arc::clone(bc);
    async move {
      let (bc_tx, bc_rx) = tokio::sync::oneshot::channel();
      bc.send(BettercapCommand::SubscribeEvents { respond_to: bc_tx })
        .await
        .unwrap_or(());

      let Ok(mut bettercap_rx) = bc_rx.await else {
        LOGGER.log_error("Agent", "Bettercap broadcast request failed");
        return;
      };

      while let Ok(msg) = bettercap_rx.recv().await {
        if tx.send(msg).await.is_err() {
          LOGGER.log_error("Agent", "Agent inbox dropped, stopping event forwarder");
          break;
        }
      }
    }
  });

  let view = Arc::clone(view);
  let epoch = Arc::clone(epoch);
  let (ev_tx, mut ev_rx) = mpsc::channel::<(String, String)>(100);
  tokio::spawn(async move {
    while let Some(msg) = rx.recv().await {
      match serde_json::from_str::<Value>(&msg) {
        Ok(jmsg) => {
          if let Some(tag) = jmsg.get("tag").and_then(|v| v.as_str())
            && tag == "wifi.client.handshake"
          {
            let _ = ev_tx.send((tag.to_string(), msg)).await;
          }
        }
        Err(e) => {
          LOGGER.log_error("Agent", &format!("Failed to parse event: {e}"));
        }
      }
    }
  });

  while let Some((tag, msg)) = ev_rx.recv().await {
    if tag == "wifi.client.handshake" {
      let json = serde_json::from_str::<Value>(&msg).unwrap_or(Value::Null);
      handle_handshake_event(&json, sm, &view, &epoch).await;
    }
  }
}

// Sorry for anyone who has to look at this
// State and Mutex gets messy reaaaally fast
pub async fn handle_handshake_event(
  jmsg: &Value,
  sm: &Arc<SessionManager>,
  view: &Arc<dyn ViewTrait + Send + Sync>,
  epoch: &Arc<RwLock<Epoch>>,
) {
  let data = jmsg.get("data");
  let ap_mac = data
    .and_then(|d| d.get("ap"))
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .to_lowercase();
  let sta_mac = data
    .and_then(|d| d.get("station"))
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .to_string();
  let filename = data
    .and_then(|d| d.get("file"))
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .to_string();
  let key = format!("{sta_mac} -> {ap_mac} ");

  let session = sm.get_session();
  let mut session_mut = session.write();

  let entry = session_mut.state.handshakes.entry(key.clone());
  match entry {
    std::collections::hash_map::Entry::Occupied(_) => {
      LOGGER.log_debug("Agent", &format!("Handshake already exists for {sta_mac} -> {ap_mac}"));
      return;
    }
    std::collections::hash_map::Entry::Vacant(entry) => {
      entry.insert(Handshake {
        mac: ap_mac.clone(),
        filename,
        timestamp: std::time::SystemTime::now(),
      });
    }
  }
  drop(session_mut);

  let last_pwned_hostname = find_ap_sta_in_session(&session, &sta_mac, &ap_mac)
    .map(|(ap, sta)| {
      LOGGER.log_info(
        "Agent",
        &format!(
          "!!! captured new handshake on channel {}, {} dBm: {} ({}) -> {} [{} ({})] !!!",
          ap.channel,
          ap.rssi,
          sta.mac,
          sta.vendor,
          hostname_or_mac(&ap),
          ap.mac,
          ap.vendor
        ),
      );
      hostname_or_mac(&ap).to_string()
    })
    .unwrap_or(ap_mac.clone());

  let mut session_mut = session.write();
  session_mut.state.last_pwned = Some(last_pwned_hostname.clone());

  let total = total_unique_handshakes(&config().bettercap.handshakes);
  let handshake_count = session_mut.state.handshakes.len();
  let mut text = format!("{handshake_count} ({total:02})");

  if let Some(last) = &session_mut.state.last_pwned {
    let _ = write!(text, " [{last}]");
  }
  drop(session_mut);

  epoch.write().track(Activity::Handshake, Some(1));
  view.set("shakes", text);
  view.on_handshakes(1);
}
