use std::sync::Arc;

use anyhow::Result;
use parking_lot::RwLock;
use tokio::task::JoinHandle;

use crate::{
  identity::Identity,
  sessions::manager::SessionManager,
  traits::{
    agent::AgentTrait, automata::AutomataTrait, bettercap::BettercapTrait, epoch::Epoch,
    events::EventBus, ui::ViewTrait,
  },
};

#[async_trait::async_trait]
pub trait Component: Dependencies + Send + Sync {
  async fn init(&mut self, ctx: &CoreModules) -> Result<()>;
  async fn start(&self) -> Result<Option<JoinHandle<()>>>;
  #[allow(unused_variables)]
  async fn stop(&self) -> Result<()> {
    Ok(())
  }
}

pub trait Dependencies: Send + Sync {
  fn name(&self) -> &'static str;
  fn dependencies(&self) -> &[&str] {
    &[]
  }
}

#[async_trait::async_trait]
pub trait CoreModule: Send + Sync {
  fn name(&self) -> &'static str;
  fn dependencies(&self) -> &[&'static str] {
    &[]
  }
}

pub trait ModuleInfo {
  fn name(&self) -> &'static str;
  fn version(&self) -> &'static str;
  fn author(&self) -> &'static str;
  fn description(&self) -> &'static str;
}

pub struct CoreModules {
  pub session_manager: Arc<SessionManager>,
  pub identity: Arc<RwLock<Identity>>,
  pub epoch: Arc<RwLock<Epoch>>,
  pub bettercap: Arc<dyn BettercapTrait + Send + Sync>,
  pub view: Arc<dyn ViewTrait + Send + Sync>,
  pub agent: Arc<dyn AgentTrait + Send + Sync>,
  pub automata: Arc<dyn AutomataTrait + Send + Sync>,
  pub events: Arc<dyn EventBus>,
}

#[async_trait::async_trait]
pub trait AdvertiserTrait: Send + Sync {
  async fn start_advertising(&self);
  async fn peer_poller(&mut self);
}
