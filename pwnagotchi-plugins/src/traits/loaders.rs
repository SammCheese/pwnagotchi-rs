use std::{error::Error, sync::Arc};

use pwnagotchi_shared::traits::general::CoreModules;

use crate::{managers::plugin_manager::PluginEntry, traits::plugins::Plugin};

pub trait PluginLoader: Send + Sync + 'static {
  fn name(&self) -> &str;
  fn extension(&self) -> &str;
  // used for *registering/getting* plugins, not initializing
  fn load_plugins(&mut self) -> Result<(), Box<dyn Error>>;
  #[allow(unused)]
  fn reload_plugin(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
    Err("Reloading individual plugins is not supported by this loader.".into())
  }
  #[allow(unused)]
  fn init_all(&mut self, core: Arc<CoreModules>) -> Result<(), Box<dyn Error>> {
    self.load_plugins()
  }
  fn unload_all(&mut self) -> Result<(), Box<dyn Error>>;
  fn disable_plugin(&mut self, name: &str) -> Result<(), Box<dyn Error>>;
  fn enable_plugin(&mut self, name: &str) -> Result<(), Box<dyn Error>>;
  fn validate_plugin(&self, _plugin: &dyn Plugin) -> bool {
    true
  }
  fn get_plugins(&self) -> &Vec<Box<PluginEntry>>;
}
