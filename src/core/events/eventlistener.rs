use std::{fmt::Write, sync::Arc};

use tokio::sync::mpsc;
use toml::Value;

use crate::core::{
  ai::Epoch,
  bettercap::BettercapCommand,
  config::config,
  log::LOGGER,
  models::net::Handshake,
  sessions::manager::SessionManager,
  traits::bettercapcontroller::BettercapController,
  ui::{state::StateValue, view::View},
  utils::total_unique_handshakes,
};

pub async fn start_event_loop(
  sm: &Arc<SessionManager>,
  bc: &Arc<dyn BettercapController>,
  epoch: &Arc<parking_lot::Mutex<Epoch>>,
  view: &Arc<View>,
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
      let json =
        serde_json::from_str::<Value>(&msg).unwrap_or_else(|_| Value::Table(toml::map::Map::new()));
      handle_handshake_event(&json, sm, &view, &epoch).await;
    }
  }
}

async fn handle_handshake_event(
  jmsg: &Value,
  sm: &Arc<SessionManager>,
  view: &Arc<View>,
  epoch: &Arc<parking_lot::Mutex<Epoch>>,
) {
  let session = sm.get_session().await;
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
  let key = format!("{ap_mac} -> {sta_mac}");

  let mut state = session.state.write(); // hold write lock once

  let entry = state.handshakes.entry(key);
  let text = match entry {
    std::collections::hash_map::Entry::Occupied(_) => {
      LOGGER.log_debug("Agent", &format!("Handshake already exists for {ap_mac} -> {sta_mac}"));
      return;
    }
    std::collections::hash_map::Entry::Vacant(entry) => {
      LOGGER
        .log_warning("Agent", &format!("!!! captured new handshake: {ap_mac} -> {sta_mac} !!!"));
      entry.insert(Handshake {
        mac: ap_mac,
        filename,
        timestamp: std::time::SystemTime::now(),
      });

      state.last_pwned = Some(sta_mac.into());
      let total = total_unique_handshakes(&config().main.handshakes_path);
      let handshake_count = state.handshakes.len();

      let mut text = format!("{handshake_count} ({total:02})");
      if let Some(last_pwned) = &state.last_pwned {
        let _ = write!(text, " [{last_pwned}]");
      }
      drop(state);
      text
    }
  };

  epoch.lock().track("handshake", Some(1));
  view.set("shakes", StateValue::Text(text));
  view.on_handshakes(1);
}
