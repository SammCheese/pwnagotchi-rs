use crate::core::{bettercap::BettercapCommand, models::bettercap::BettercapSession};

#[async_trait::async_trait]
pub trait BettercapController: Send + Sync {
  async fn send(&self, cmd: BettercapCommand) -> anyhow::Result<()>;
  async fn session(&self) -> anyhow::Result<Option<BettercapSession>>;
}
