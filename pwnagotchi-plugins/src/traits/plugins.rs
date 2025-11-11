use std::{
  error::Error,
  sync::{Arc, LazyLock},
};

use parking_lot::{RawRwLock, lock_api::RwLockReadGuard};
use pwnagotchi_shared::{config::Config, logger::Log, traits::general::CoreModules};

use crate::traits::{events::DynamicEventAPITrait, hooks::DynamicHookAPITrait};

pub struct PluginInfo {
  pub name: &'static str,
  pub version: &'static str,
  pub author: &'static str,
  pub description: &'static str,
  pub license: &'static str,
}

pub struct PluginAPI<'a> {
  pub hook_api: &'a mut dyn DynamicHookAPITrait,
  pub event_api: &'a mut dyn DynamicEventAPITrait,
  pub core_modules: Arc<CoreModules>,
  pub logger: Arc<&'a LazyLock<Log>>,
  pub config: &'a RwLockReadGuard<'static, RawRwLock, Config>,
}

unsafe impl Send for PluginAPI<'_> {}
unsafe impl Sync for PluginAPI<'_> {}

pub trait Plugin: Send + Sync + 'static {
  fn info(&self) -> &PluginInfo;
  fn webhook(&self) -> Option<&'static str> {
    None
  }
  fn on_load(&mut self, api: PluginAPI) -> Result<(), Box<dyn Error>>;
  fn on_unload(&mut self) -> Result<(), Box<dyn Error>>;
  fn get_template(&self) -> Option<&'static str> {
    None
  }
}

#[async_trait::async_trait]
pub trait AsyncPlugin: Send + Sync {
  fn info(&self) -> &PluginInfo;
  async fn on_load(&mut self, api: &PluginAPI) -> Result<(), Box<dyn Error>>;
  async fn on_unload(&mut self) -> Result<(), Box<dyn Error>>;
}
