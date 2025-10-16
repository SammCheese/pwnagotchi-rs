use std::{error::Error, sync::Arc};

use pwnagotchi_shared::traits::general::CoreModules;

use crate::traits::{
  hooks::DynamicHookAPITrait,
  plugins::{Plugin, PluginInfo},
};

#[derive(Default)]
pub struct HelloWorld;

impl HelloWorld {
  pub fn new() -> Self {
    Self {}
  }
}

impl Plugin for HelloWorld {
  fn info(&self) -> &PluginInfo {
    &PluginInfo {
      name: "HelloWorld",
      version: "0.1.0",
      author: "Your Name",
      description: "A simple Hello World plugin",
    }
  }

  fn on_load(
    &mut self,
    _hook_api: &mut dyn DynamicHookAPITrait,
    _core: Arc<CoreModules>,
  ) -> Result<(), Box<dyn Error + 'static>> {
    println!("Hello, world! Plugin started.");
    Ok(())
  }

  fn on_unload(&mut self) -> Result<(), Box<dyn Error + 'static>> {
    println!("Goodbye, world! Plugin shutting down.");
    Ok(())
  }
}
