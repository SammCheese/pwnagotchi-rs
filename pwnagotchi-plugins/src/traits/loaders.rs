use std::error::Error;

use crate::traits::plugins::Plugin;

pub trait PluginLoader: Send + Sync + 'static {
  fn name(&self) -> &str;
  fn extension(&self) -> &str;
  fn load_plugins(&mut self);
  fn take_plugins(&mut self) -> Vec<Box<dyn Plugin>>;
  #[allow(unused)]
  fn reload_plugin(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
    Err("Reloading individual plugins is not supported by this loader.".into())
  }
  fn validate_plugin(&self, _plugin: &dyn Plugin) -> bool {
    true
  }
}
