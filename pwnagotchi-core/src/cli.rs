use std::{sync::Arc, time::Duration};

use anyhow::Result;
use pwnagotchi_macros::hookable;
use pwnagotchi_shared::{
  logger::LOGGER,
  models::agent::RunningMode,
  sessions::manager::SessionManager,
  traits::{
    agent::AgentTrait,
    automata::AutomataTrait,
    bettercap::BettercapTrait,
    general::{Component, CoreModules, Dependencies},
  },
};
use tokio::{task::JoinHandle, time::sleep};

pub struct CliComponent {
  cli: Option<Arc<Cli>>,
  manual: bool,
}

impl Dependencies for CliComponent {
  fn name(&self) -> &'static str {
    "CliComponent"
  }
  fn dependencies(&self) -> &[&str] {
    &[
      "SessionManager",
      "Bettercap",
      "Agent",
      "Automata",
    ]
  }
}

#[async_trait::async_trait]
impl Component for CliComponent {
  async fn init(&mut self, ctx: &CoreModules) -> Result<()> {
    let (sm, bc, agent, observer) =
      (&ctx.session_manager, &ctx.bettercap, &ctx.agent, &ctx.automata);

    let sm = Arc::clone(sm);
    let bc = Arc::clone(bc);
    let agent = Arc::clone(agent);
    let observer = Arc::clone(observer);

    let cli = Cli::new(sm, bc, agent, observer, self.manual);

    self.cli = Some(Arc::new(cli));

    Ok(())
  }

  async fn start(&self) -> Result<Option<JoinHandle<()>>> {
    if let Some(cli) = &self.cli {
      let cli = Arc::clone(cli);
      let handle = tokio::spawn(async move {
        if cli.manual {
          cli.do_manual_mode().await;
        } else {
          cli.do_auto_mode().await;
        }
      });
      Ok(Some(handle))
    } else {
      Ok(None)
    }
  }
}

impl CliComponent {
  pub fn new(manual: bool) -> Self {
    Self { cli: None, manual }
  }
}

pub struct Cli {
  pub sm: Arc<SessionManager>,
  pub bc: Arc<dyn BettercapTrait + Send + Sync>,
  pub agent: Arc<dyn AgentTrait + Send + Sync>,
  pub observer: Arc<dyn AutomataTrait + Send + Sync>,
  pub manual: bool,
}

#[hookable]
impl Cli {
  pub fn new(
    sm: Arc<SessionManager>,
    bc: Arc<dyn BettercapTrait + Send + Sync>,
    agent: Arc<dyn AgentTrait + Send + Sync>,
    observer: Arc<dyn AutomataTrait + Send + Sync>,
    manual: bool,
  ) -> Self {
    Self { sm, bc, agent, observer, manual }
  }

  pub async fn do_auto_mode(&self) {
    LOGGER.log_info("Pwnagotchi", "Starting auto mode...");

    self.agent.set_mode(RunningMode::Auto).await;
    self.agent.start_pwnagotchi();

    loop {
      self.agent.recon().await;

      let aps = self.agent.get_access_points_by_channel().await;

      for (ch, aps) in aps {
        sleep(Duration::from_secs(1)).await;
        self.agent.set_channel(ch).await;

        if !self.observer.is_stale() && self.observer.any_activity() {
          LOGGER.log_info("Pwnagotchi", format!("{} APs on channel {ch}", aps.len()).as_str());
        }

        for ap in aps {
          self.agent.associate(&ap, None).await;

          for sta in &ap.clients {
            self.agent.deauth(&ap, sta, None).await;
            // Shoo Nexmon Bugs!
            sleep(Duration::from_secs(1)).await;
          }
        }
      }

      self.observer.next_epoch();
    }
  }

  #[hookable]
  pub async fn do_manual_mode(&self) {
    LOGGER.log_info("Pwnagotchi", "Starting in manual mode...");
    self.agent.set_mode(RunningMode::Manual).await;

    loop {
      sleep(Duration::from_secs(60)).await;
    }
  }

  #[hookable]
  pub async fn do_custom_mode(&self, _mode: &str) {
    LOGGER.log_info("Pwnagotchi", "Starting in custom mode...");
    self.agent.set_mode(RunningMode::Custom).await;

    loop {
      sleep(Duration::from_secs(60)).await;
    }
  }

  #[hookable]
  pub async fn do_ai_mode(&self) {
    LOGGER.log_info("Pwnagotchi", "Starting in AI mode...");
    self.agent.set_mode(RunningMode::Ai).await;

    loop {
      sleep(Duration::from_secs(60)).await;
    }
  }
}
