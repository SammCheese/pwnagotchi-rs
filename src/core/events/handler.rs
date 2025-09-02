use std::{fmt::Write, sync::Arc};

use serde_json::Value;

use crate::core::{
  commands::AgentHandle, config::config, log::LOGGER, models::net::Handshake,
  ui::state::StateValue, utils::total_unique_handshakes,
};

#[derive(Clone)]

pub struct EventHandler {
  pub agent: Arc<AgentHandle>,
}

impl EventHandler {
  pub const fn new(agent: Arc<AgentHandle>) -> Self {
    Self { agent }
  }

  pub async fn on_event_async(&self, raw: String) {
    LOGGER.log_debug("Agent", &format!("Received event: {raw}"));

    match serde_json::from_str::<Value>(&raw) {
      Ok(jmsg) => {
        if let Some(tag) = jmsg.get("tag").and_then(|v| v.as_str())
          && tag == "wifi.client.handshake"
        {
          self.handle_handshake_event(&jmsg).await;
        }
      }
      Err(e) => {
        LOGGER.log_error("Agent", &format!("Failed to parse event: {e}"));
      }
    }
  }

  async fn handle_handshake_event(&self, jmsg: &Value) {
    LOGGER.log_info("BIG EVENT", format!("{jmsg}").as_str());
    let data = jmsg.get("data").unwrap_or_default();
    let ap_mac = data.get("ap").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
    let sta_mac = data.get("station").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let filename = data.get("file").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let key = format!("{ap_mac} -> {sta_mac}");

    self
      .agent
      .execute(move |agent| match agent.handshakes.entry(key) {
        std::collections::hash_map::Entry::Occupied(_) => {
          LOGGER.log_debug("Agent", &format!("Handshake already exists for {ap_mac} -> {sta_mac}"));
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
          LOGGER.log_warning(
            "Agent",
            &format!("!!! captured new handshake: {ap_mac} -> {sta_mac} !!!"),
          );

          entry.insert(Handshake {
            mac: ap_mac,
            filename,
            timestamp: std::time::SystemTime::now(),
          });

          agent.last_pwned = Some(sta_mac);
          agent.automata.epoch.track("handshake", Some(1));

          let total = total_unique_handshakes(&config().main.handshakes_path);
          let current = agent.handshakes.len();
          let mut text = format!("{current} ({total:02})");

          if let Some(last_pwned) = &agent.last_pwned {
            let _ = write!(text, " [{last_pwned}]");
          }

          agent.automata.view.set("shakes", StateValue::Text(text));
          agent.automata.view.on_handshakes(1);
        }
      })
      .await;
  }
}
