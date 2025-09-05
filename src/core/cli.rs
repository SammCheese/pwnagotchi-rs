use std::{sync::Arc, time::Duration};

use tokio::time::sleep;

use crate::core::{
  agent::{Agent, RunningMode},
  automata::Automata,
  log::LOGGER,
  sessions::manager::SessionManager,
  setup,
  traits::{agentobserver::AgentObserver, bettercapcontroller::BettercapController},
};

trait PwnagotchiMode {
  fn mode(&self) -> RunningMode;
}

pub async fn do_auto_mode(
  sm: &Arc<SessionManager>,
  bc: &Arc<dyn BettercapController>,
  agent: &Arc<Agent>,
  observer: Arc<Automata>,
) {
  LOGGER.log_info("Pwnagotchi", "Starting auto mode...");

  // Set mode and perform internal setup
  agent.set_mode(sm, RunningMode::Auto).await;
  setup::perform_bettercap_setup(bc).await;
  agent.start();

  loop {
    agent.recon(sm).await;

    let aps = agent.get_access_points_by_channel(sm).await;

    for (ch, aps) in aps {
      sleep(Duration::from_secs(1)).await;
      agent.set_channel(sm, ch).await;

      if !observer.is_stale() && observer.any_activity() {
        LOGGER.log_info("Pwnagotchi", format!("{} APs on channel {ch}", aps.len()).as_str());
      }

      for ap in aps {
        agent.associate(sm, &ap, None).await;

        for sta in &ap.clients {
          agent.deauth(sm, &ap, sta, None).await;
          // Shoo Nexmon Bugs!
          sleep(Duration::from_secs(1)).await;
        }
      }
    }

    observer.next_epoch();
  }
}

pub async fn do_manual_mode(sm: &Arc<SessionManager>, agent: &Arc<Agent>) {
  LOGGER.log_info("Pwnagotchi", "Starting in manual mode...");

  agent.set_mode(sm, RunningMode::Manual).await;

  loop {
    sleep(Duration::from_secs(60)).await;
  }
}
