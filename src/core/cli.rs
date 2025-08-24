use std::time::Duration;
//use std::thread::sleep;
use tokio::time::sleep;

use crate::core::{ agent::Agent, log::LOGGER };

#[allow(clippy::future_not_send)]
pub async fn do_auto_mode(agent: &mut Agent) {
    LOGGER.log_info("Pwnagotchi", "Starting auto mode...");

    agent.mode = "auto".to_string();
    agent.start().await;

    loop {
        agent.drain_events();
        agent.recon().await;
        let channels = agent.get_access_points_by_channel().await;
        LOGGER.log_info(
            "Pwnagotchi",
            &format!("Found {} channels with access points", channels.len())
        );

        for (ch, aps) in channels {
            sleep(Duration::from_secs(5)).await;
            agent.drain_events();
            agent.set_channel(ch).await;

            if !agent.automata.is_stale() && agent.automata.any_activity() {
                LOGGER.log_info(
                    "Pwnagotchi",
                    &format!("{} access points on channel {}", aps.len(), ch)
                );
            }

            for ap in aps {
                agent.associate(&ap, None).await;
                for sta in &ap.clients {
                    agent.deauth(&ap, sta, None).await;
                    sleep(Duration::from_secs(1)).await;
                    agent.drain_events();
                }
            }
        }

        agent.automata.next_epoch();
    }
}
