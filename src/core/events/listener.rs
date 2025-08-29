use std::sync::{ Arc, Mutex as StdMutex };

use tokio::sync::{mpsc, Mutex};
use crate::core::{ agent::Agent, events::handler::EventHandler, log::LOGGER };

pub struct EventListener {
  pub agent: Arc<Mutex<Agent>>,
  pub handler: EventHandler,
}

impl EventListener {
  pub fn new(agent: Arc<Mutex<Agent>>) -> Self {
    let handler = EventHandler::new(Arc::clone(&agent));
    Self {
      agent,
      handler,
    }
  }

  pub fn start_event_loop(self) {
    LOGGER.log_debug("Agent", "Starting event loop...");

    LOGGER.log_debug("EVENT", "Got Agent Lock!");

    let (tx, mut rx) = mpsc::channel::<String>(1000);

    tokio::spawn(async move {
      let mut bc_rx = {
        let agent = self.agent.lock().await;
        agent.bettercap.subscribe_events()
      };
      loop {
        match bc_rx.recv().await {
          Ok(msg) => {
            LOGGER.log_debug("Agent", &format!("Received Bettercap event: {msg}"));
            if tx.send(msg).await.is_err() {
              LOGGER.log_error("Agent", "Agent inbox dropped, stopping event forwarder");
              break;
            }
          }
          Err(e) => {
            LOGGER.log_warning("Agent", &format!("Bettercap event channel error: {e}"));
            break;
          }
        }
      }
    });

    let handler = self.handler;

    tokio::spawn(async move {
      while let Some(msg) = rx.recv().await {
        handler.on_event_async(msg).await;
      }
    });
  }
}
