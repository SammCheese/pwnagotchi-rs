use std::sync::{Arc};
use tokio::sync::{Mutex};

use serde_json::Value;

use crate::core::{agent::Agent, log::LOGGER};

#[derive(Clone)]
pub struct EventHandler {
  pub agent: Arc<Mutex<Agent>>,
}

impl EventHandler {
  pub const fn new(agent: Arc<Mutex<Agent>>) -> Self {
    Self { agent }
  }

  pub async fn on_event_async(&self, raw: String) {
    match serde_json::from_str::<Value>(&raw) {
      Ok(jmsg) => {
        if let Some(tag) = jmsg.get("tag").and_then(|v| v.as_str())
          && tag == "wifi.client.handshake" {
            self.handle_handshake_event(jmsg).await;
          }
      }
      Err(e) => {
        LOGGER.log_error("Agent", &format!("Failed to parse event: {e}"));
      }
    }
  }

  async fn handle_handshake_event(&self, jmsg: Value) {
    let ap_mac = jmsg
      .get("ap")
      .and_then(|v| v.as_str())
      .unwrap_or("")
      .to_lowercase();
    let sta_mac = jmsg
      .get("station")
      .and_then(|v| v.as_str())
      .unwrap_or("");
    let key = format!("{ap_mac} -> {sta_mac}");

    let mut agent = self.agent.lock().await;
    if agent.handshakes.contains_key(&key) {
      LOGGER.log_debug("Agent", &format!("Handshake already exists for {key}"));
    } else {
      LOGGER.log_warning("Agent", &format!("New handshake captured: {key}"));
      agent.last_pwned = Some(ap_mac);
      agent.update_handshakes(1);
    }
  }
}
