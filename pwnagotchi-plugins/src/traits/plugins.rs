use std::{error::Error, sync::Arc};

use pwnagotchi_shared::traits::general::CoreModules;

use crate::traits::{events::DynamicEventAPITrait, hooks::DynamicHookAPITrait};

pub struct PluginInfo {
  pub name: &'static str,
  pub version: &'static str,
  pub author: &'static str,
  pub description: &'static str,
  pub license: &'static str,
}

pub trait Plugin: Send + Sync + 'static {
  fn info(&self) -> &PluginInfo;
  fn webhook(&self) -> Option<&'static str> {
    None
  }
  fn on_load(
    &mut self,
    hook_api: &mut dyn DynamicHookAPITrait,
    event_api: &mut dyn DynamicEventAPITrait,
    core: Arc<CoreModules>,
  ) -> Result<(), Box<dyn Error>>;
  fn on_unload(&mut self) -> Result<(), Box<dyn Error>>;
}

#[async_trait::async_trait]
pub trait AsyncPlugin: Send + Sync {
  fn info(&self) -> &PluginInfo;
  async fn on_load(
    &mut self,
    hook_api: &mut dyn DynamicHookAPITrait,
    event_api: &mut dyn DynamicEventAPITrait,
  ) -> Result<(), Box<dyn Error>>;
  async fn on_unload(&mut self) -> Result<(), Box<dyn Error>>;
}
