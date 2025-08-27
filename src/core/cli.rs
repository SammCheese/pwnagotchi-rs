use std::{ sync::Arc, time::Duration };
//use std::thread::sleep;
use tokio::{ sync::Mutex, time::sleep };

use crate::core::{ agent::{ Agent, RunningMode }, log::LOGGER };

#[allow(clippy::future_not_send)]
pub async fn do_auto_mode(agent: Arc<Mutex<Agent>>) {
  LOGGER.log_info("Pwnagotchi", "Starting auto mode...");

  {
    let mut a = agent.lock().await;
    a.mode = RunningMode::Auto;
    a.start().await;
    drop(a);
  }

  loop {
    let channels = {
      let mut a = agent.lock().await;
      a.recon().await;
      a.get_access_points_by_channel().await
    };
    LOGGER.log_info("Pwnagotchi", &format!("Found {} channels with access points", channels.len()));

    for (ch, aps) in channels {
      {
        agent.lock().await.set_channel(ch).await;
      }
      sleep(Duration::from_secs(5)).await;

      {
        let mut a = agent.lock().await;
        if !a.automata.is_stale() && a.automata.any_activity() {
          LOGGER.log_info("Pwnagotchi", &format!("{} APs on channel {}", aps.len(), ch));
        }
      }

      for ap in aps {
        {
          agent.lock().await.associate(&ap, None).await;
        }
        for sta in &ap.clients {
          {
            agent.lock().await.deauth(&ap, sta, None).await;
          }
          sleep(Duration::from_secs(1)).await;
        }
      }
    }

    {
      let mut a = agent.lock().await;
      a.automata.next_epoch();
    }
  }
}

pub async fn do_manual_mode(agent: Arc<Mutex<Agent>>, skip: Option<bool>) {
  LOGGER.log_info("Pwnagotchi", "Starting in manual mode...");

  {
    let mut a = agent.lock().await;
    a.mode = RunningMode::Manual;
  }
  {
    let mut lastsession = agent.lock().await.lastsession.clone();
    let a = agent.lock().await;
    lastsession.parse(&a.view, skip);
  }

  loop {
    sleep(Duration::from_secs(60)).await;
  }
}
