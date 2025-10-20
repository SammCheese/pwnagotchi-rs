use std::{error::Error, sync::Arc};

use pwnagotchi_plugins::traits::{
  hooks::DynamicHookAPITrait,
  plugins::{Plugin, PluginInfo},
};
use pwnagotchi_shared::traits::general::CoreModules;

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

#[allow(improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn _plugin_create() -> *mut dyn Plugin {
  let plugin: Box<dyn Plugin> = Box::new(HelloWorld::new());
  Box::into_raw(plugin)
}

#[allow(improper_ctypes_definitions)]
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _plugin_destroy(ptr: *mut dyn Plugin) {
  println!("Destroying HelloWorld plugin");
  if !ptr.is_null() {
    unsafe {
      drop(Box::from_raw(ptr));
    }
  }
}
