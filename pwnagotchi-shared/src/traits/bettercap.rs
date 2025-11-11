use tokio::sync::broadcast;

use crate::{models::bettercap::BettercapSession, traits::general::CoreModule};

pub enum BettercapCommand {
  Run { cmd: String, respond_to: tokio::sync::oneshot::Sender<Result<(), anyhow::Error>> },
  GetSession { respond_to: tokio::sync::oneshot::Sender<Option<BettercapSession>> },
  SubscribeEvents { respond_to: tokio::sync::oneshot::Sender<broadcast::Receiver<String>> },
}

impl BettercapCommand {
  pub fn run<S>(
    cmd: S,
    respond_to: Option<tokio::sync::oneshot::Sender<Result<(), anyhow::Error>>>,
  ) -> Self
  where
    S: 'static + AsRef<str>,
  {
    let tx = respond_to.unwrap_or_else(|| {
      let (tx, rx) = tokio::sync::oneshot::channel();
      drop(rx);
      tx
    });
    Self::Run { cmd: cmd.as_ref().into(), respond_to: tx }
  }

  pub fn run_fire_and_forget<S>(cmd: S) -> Self
  where
    S: 'static + AsRef<str>,
  {
    Self::run(cmd, None)
  }
}

#[async_trait::async_trait]
pub trait BettercapTrait: Send + Sync + CoreModule {
  async fn send(&self, cmd: BettercapCommand) -> anyhow::Result<()>;
  async fn session(&self) -> anyhow::Result<Option<BettercapSession>>;
  async fn run_websocket(&self);
  fn is_ready(&self) -> bool;
  async fn run(&self, cmd: &str) -> Result<(), anyhow::Error>;

  async fn run_fire_and_forget(&self, cmd: String) -> anyhow::Result<()> {
    self.send(BettercapCommand::run_fire_and_forget(cmd)).await
  }
}

#[async_trait::async_trait]
pub trait SetupTrait: Send + Sync {
  async fn perform_setup(&self);
}
