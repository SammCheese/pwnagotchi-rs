use std::sync::Arc;

use crate::core::{
  bettercap::{BettercapCommand, BettercapHandle},
  commands::AgentHandle,
  events::handler::EventHandler,
  log::LOGGER,
};

pub fn start_event_loop(agent: &Arc<AgentHandle>, bc: &Arc<BettercapHandle>) {
  let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(1000);

  let handler = EventHandler::new(Arc::clone(agent));

  tokio::spawn({
    let tx = tx;
    let bc = Arc::clone(bc);
    async move {
      let (bc_tx, bc_rx) = tokio::sync::oneshot::channel();
      bc.send_command(BettercapCommand::SubscribeEvents { respond_to: bc_tx })
        .await;

      let Ok(mut bettercap_rx) = bc_rx.await else {
        LOGGER.log_error("Agent", "Bettercap broadcast request failed");
        return;
      };

      while let Ok(msg) = bettercap_rx.recv().await {
        LOGGER.log_debug("Agent", &format!("Received Bettercap event: {msg}"));
        if tx.send(msg).await.is_err() {
          LOGGER.log_error("Agent", "Agent inbox dropped, stopping event forwarder");
          break;
        }
      }
    }
  });

  tokio::spawn(async move {
    while let Some(msg) = rx.recv().await {
      handler.on_event_async(msg).await;
    }
  });
}
