use std::{fmt::Write, sync::Arc};

use pwnagotchi_shared::{
  config::config,
  log::LOGGER,
  models::net::Handshake,
  sessions::manager::SessionManager,
  traits::ui::ViewTrait,
  types::{epoch::Activity, ui::StateValue},
  utils::general::{hostname_or_mac, total_unique_handshakes},
};
use serde_json::Value;
use tokio::sync::mpsc;

use crate::{
  agent::find_ap_sta_in_session, ai::Epoch, bettercap::BettercapCommand,
  traits::bettercapcontroller::BettercapController,
};

pub async fn start_event_loop(
  sm: &Arc<SessionManager>,
  bc: &Arc<dyn BettercapController>,
  epoch: &Arc<parking_lot::Mutex<Epoch>>,
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
  epoch: &Arc<parking_lot::Mutex<Epoch>>,
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

  let session = sm.get_session().await;
  let mut state = session.state.write();

  let entry = state.handshakes.entry(key.clone());
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
  drop(state);

  let state = session.state.read();
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
  drop(state);

  let mut state = session.state.write();
  state.last_pwned = Some(last_pwned_hostname.clone());

  let total = total_unique_handshakes(&config().bettercap.handshakes);
  let handshake_count = state.handshakes.len();
  let mut text = format!("{handshake_count} ({total:02})");

  if let Some(last) = &state.last_pwned {
    let _ = write!(text, " [{last}]");
  }
  drop(state);

  epoch.lock().track(Activity::Handshake, Some(1));
  view.set("shakes", StateValue::Text(text));
  view.on_handshakes(1);
}
