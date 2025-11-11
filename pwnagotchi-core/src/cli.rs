use std::{sync::Arc, time::Duration};

use anyhow::Result;
use pwnagotchi_macros::hookable;
use pwnagotchi_shared::{
  logger::LOGGER, models::agent::RunningMode, sessions::session_stats::SessionStats,
  traits::general::CoreModules, types::events::EventPayload,
};
use tokio::time::sleep;

pub struct Cli {
  pub core: Arc<CoreModules>,
}

#[hookable]
impl Cli {
  pub fn new(core: Arc<CoreModules>) -> Self {
    Self { core }
  }

  pub async fn do_auto_mode(&self) {
    LOGGER.log_info("Pwnagotchi", "Starting auto mode...");

    self.core.agent.set_mode(RunningMode::Auto).await;
    self.core.session_manager.get_last_session().write().reparse();
    self.core.agent.start_pwnagotchi();

    loop {
      self.core.agent.recon().await;

      let aps = self.core.agent.get_access_points_by_channel().await;

      for (ch, aps) in aps {
        sleep(Duration::from_secs(1)).await;
        self.core.agent.set_channel(ch).await;

        if !self.core.automata.is_stale() && self.core.automata.any_activity() {
          LOGGER.log_info("Pwnagotchi", format!("{} APs on channel {ch}", aps.len()).as_str());
        }

        for ap in aps {
          self.core.agent.associate(&ap, None).await;

          for sta in &ap.clients {
            self.core.agent.deauth(&ap, sta, None).await;
            // Shoo Nexmon Bugs!
            sleep(Duration::from_secs(1)).await;
          }
        }
      }

      self.core.automata.next_epoch();

      if self.core.grid.is_connected() {
        let last_session = &self.core.session_manager.get_last_session();

        let Some(stats) = last_session.read().stats.clone() else {
          LOGGER.log_debug("GRID", "No session stats available to upload.");
          continue;
        };

        let _ = self
          .core
          .events
          .emit_payload("internet_available", EventPayload::new::<SessionStats>(&stats).unwrap())
          .await;
        drop(stats);
      }
    }
  }

  pub async fn do_manual_mode(&self) {
    LOGGER.log_info("Pwnagotchi", "Starting in manual mode...");
    self.core.agent.set_mode(RunningMode::Manual).await;
    self.core.session_manager.get_last_session().write().reparse();

    loop {
      let last_session = &self.core.session_manager.get_last_session();
      self.core.view.on_manual_mode(&last_session.read());

      if self.core.grid.is_connected() {
        if last_session.read().stats.is_none() {
          LOGGER.log_debug("Pwnagotchi", "No session stats available to upload.");
          continue;
        }
        let stats = &last_session.read().stats.clone();
        let _ = self
          .core
          .events
          .emit_payload(
            "internet_available",
            EventPayload::new::<SessionStats>(&stats.clone().unwrap()).unwrap(),
          )
          .await;
      }
      sleep(Duration::from_secs(60)).await;
    }
  }

  pub async fn do_custom_mode(&self, _mode: &str) {
    LOGGER.log_info("Pwnagotchi", "Starting in custom mode...");
    self.core.agent.set_mode(RunningMode::Custom).await;

    loop {
      sleep(Duration::from_secs(60)).await;
    }
  }

  pub async fn do_ai_mode(&self) {
    LOGGER.log_info("Pwnagotchi", "Starting in AI mode...");
    self.core.agent.set_mode(RunningMode::Ai).await;

    loop {
      sleep(Duration::from_secs(60)).await;
    }
  }
}
