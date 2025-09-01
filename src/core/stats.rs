use std::sync::Arc;

use tokio::{sync::oneshot, time::sleep};

use crate::core::{
    commands::AgentHandle,
    config::config,
    ui::state::StateValue,
    utils::{format_duration_human, total_unique_handshakes},
};

#[derive(Default)]
pub struct SessionFetcher {}

impl SessionFetcher {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn start_sessionfetcher(&self, agent: Arc<AgentHandle>) {
        tokio::spawn(async move {
            loop {
                update_uptime(&agent).await;
                update_aps(&agent).await;
                update_handshakes(&agent).await;
                sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    }
}

async fn update_handshakes(agent: &AgentHandle) {
    // Handshakes
    agent
        .execute(|agent| {
            let total = total_unique_handshakes(&config().main.handshakes_path);
            let current = agent.handshakes.len();
            let mut text = format!("{current} ({total})");
            if let Some(last_pwned) = &agent.last_pwned {
                use std::fmt::Write;
                let _ = write!(text, " [{last_pwned}]");
            }
            agent.automata.view.set("shakes", StateValue::Text(text));
        })
        .await;
}

async fn update_aps(agent: &AgentHandle) {
    agent
        .execute(|agent| {
            let tot_aps = agent.access_points.len();
            let tot_stas: usize = agent.access_points.iter().map(|ap| ap.clients.len()).sum();

            if agent.current_channel == 0 {
                agent
                    .automata
                    .view
                    .set("aps", StateValue::Text(tot_aps.to_string()));
                agent
                    .automata
                    .view
                    .set("sta", StateValue::Text(tot_stas.to_string()));
            } else {
                let aps_on_channel = agent.get_aps_on_channel(agent.current_channel);
                let stas_on_channel: usize = aps_on_channel.iter().map(|ap| ap.clients.len()).sum();
                agent
                    .automata
                    .view
                    .set("aps", StateValue::Text(format!("{} ({})", aps_on_channel.len(), tot_aps)));
                agent
                    .automata
                    .view
                    .set("sta", StateValue::Text(format!("{stas_on_channel} ({tot_stas})")));
            }
        })
        .await;
}

async fn update_uptime(agent: &AgentHandle) {
    let (tx, rx) = oneshot::channel();
    // Uptime
    agent
        .execute(|agent| {
            let _ = tx.send(agent.started_at);
        })
        .await;
    let now = std::time::SystemTime::now();

    if let Ok(started) = rx.await {
        agent
            .execute(move |agent| {
                agent.automata.view.set(
                    "uptime",
                    StateValue::Text(format_duration_human(
                        now.duration_since(started).unwrap_or_default(),
                    )),
                );
            })
            .await;
    }
}
