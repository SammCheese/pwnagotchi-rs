use std::{error::Error, sync::Arc};

use pwnagotchi_shared::traits::general::CoreModules;

use crate::traits::hooks::DynamicHookAPITrait;

pub struct PluginInfo {
  pub name: &'static str,
  pub version: &'static str,
  pub author: &'static str,
  pub description: &'static str,
}

pub trait Plugin: Send + Sync + 'static {
  fn info(&self) -> &PluginInfo;
  fn on_load(
    &mut self,
    hook_api: &mut dyn DynamicHookAPITrait,
    core: Arc<CoreModules>,
  ) -> Result<(), Box<dyn Error>>;
  fn on_unload(&mut self) -> Result<(), Box<dyn Error>>;
}

#[async_trait::async_trait]
pub trait AsyncPlugin: Send + Sync {
  fn info(&self) -> &PluginInfo;
  async fn on_load(&mut self, hook_api: &mut dyn DynamicHookAPITrait)
  -> Result<(), Box<dyn Error>>;
  async fn on_unload(&mut self) -> Result<(), Box<dyn Error>>;
}
