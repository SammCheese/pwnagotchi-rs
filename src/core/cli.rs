use std::{sync::Arc, time::Duration};
use tokio::{sync::oneshot, time::sleep};

use crate::core::{
    agent::RunningMode,
    commands::{AgentCommand, AgentHandle},
    log::LOGGER,
};

pub async fn do_auto_mode(agent: Arc<AgentHandle>, skip: Option<bool>) {
    LOGGER.log_info("Pwnagotchi", "Starting auto mode...");

    // Set mode and perform internal setup
    agent
        .send_command(AgentCommand::SetMode {
            mode: RunningMode::Auto,
        })
        .await;

    agent
        .send_command(AgentCommand::ParseLastSession { skip })
        .await;
    agent.send_command(AgentCommand::Start).await;

    loop {
        // Start Reconning
        agent.send_command(AgentCommand::Recon).await;

        let (tx, rx) = oneshot::channel();

        // Fetch channels and APs inside them
        agent
            .send_command(AgentCommand::GetAccessPointsByChannel { respond_to: tx })
            .await;

        for (ch, aps) in rx.await.unwrap_or_default() {
            // Dont Spam just yet...
            sleep(Duration::from_secs(1)).await;

            // Set the channel
            agent
                .send_command(AgentCommand::SetChannel { channel: ch })
                .await;

            sleep(Duration::from_secs(5)).await;
            LOGGER.log_info(
                "Pwnagotchi",
                &format!("{} APs on channel {}", aps.len(), ch),
            );

            for ap in aps {
                let ap_clone = ap.clone();
                // Associate to every AP
                agent
                    .send_command(AgentCommand::Associate {
                        ap: ap_clone.clone(),
                        throttle: None,
                    })
                    .await;
                for sta in ap.clients {
                    // Deauth everyone!
                    agent
                        .send_command(AgentCommand::Deauth {
                            ap: Box::new(ap_clone.clone()),
                            sta,
                            throttle: None,
                        })
                        .await;
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }

        agent
            .execute(|agent| {
                agent.automata.next_epoch();
            })
            .await;
    }
}

pub async fn do_manual_mode(agent: Arc<AgentHandle>, skip: Option<bool>) {
    LOGGER.log_info("Pwnagotchi", "Starting in manual mode...");

    agent
        .send_command(AgentCommand::SetMode {
            mode: RunningMode::Manual,
        })
        .await;
    agent
        .send_command(AgentCommand::ParseLastSession { skip })
        .await;

    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
