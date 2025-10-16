use crate::{
  examples::{awesome_hooking::AwesomeHooking, hello_world::HelloWorld},
  traits::{loaders::PluginLoader, plugins::Plugin},
};

pub struct RustPluginLoader {
  plugins: Vec<Box<dyn Plugin>>,
}

impl Default for RustPluginLoader {
  fn default() -> Self {
    Self::new()
  }
}

impl RustPluginLoader {
  pub fn new() -> Self {
    Self { plugins: Vec::new() }
  }
}

impl PluginLoader for RustPluginLoader {
  fn name(&self) -> &str {
    "RustPluginLoader"
  }

  fn load_plugins(&mut self) {
    self.plugins.push(Box::new(AwesomeHooking::new()));
    self.plugins.push(Box::new(HelloWorld::new()));
  }

  // Used to take the Plugins out of the loader and into the PluginManager
  fn take_plugins(&mut self) -> Vec<Box<dyn Plugin>> {
    std::mem::take(&mut self.plugins)
  }
}
