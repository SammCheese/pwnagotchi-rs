use std::{error::Error, sync::Arc};

use pwnagotchi_plugins::traits::{
  events::{AsyncEventHandler, DynamicEventAPITrait, EventHandler},
  hooks::DynamicHookAPITrait,
  plugins::{Plugin, PluginInfo},
};
use pwnagotchi_shared::{traits::general::CoreModules, types::events::EventPayload};

#[derive(Default)]
pub struct HelloWorld;

impl HelloWorld {
  pub fn new() -> Self {
    Self {}
  }

  fn handle_ready_event(&self) -> EventHandler {
    Arc::new(move |_payload: &EventPayload| {
      eprintln!("Ready event received!");
      Ok(())
    })
  }

  fn handle_starting_event(&self) -> EventHandler {
    Arc::new(move |_payload: &EventPayload| {
      eprintln!("Starting event received!");
      Ok(())
    })
  }

  fn handle_plugin_event(&self) -> EventHandler {
    Arc::new(move |_payload: &EventPayload| {
      eprintln!("plugin init event received!");
      Ok(())
    })
  }
}

impl Plugin for HelloWorld {
  fn info(&self) -> &PluginInfo {
    &PluginInfo {
      name: "HelloWorld",
      version: "0.1.0",
      author: "Your Name",
      description: "A simple Hello World plugin",
      license: "MIT",
    }
  }

  fn on_load(
    &mut self,
    _hook_api: &mut dyn DynamicHookAPITrait,
    event_api: &mut dyn DynamicEventAPITrait,
    _core: Arc<CoreModules>,
  ) -> Result<(), Box<dyn Error + 'static>> {
    event_api.register_listener("ready", self.handle_ready_event())?;
    event_api.register_listener("starting", self.handle_starting_event())?;
    event_api.register_listener("plugin::init", self.handle_plugin_event())?;
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
  if !ptr.is_null() {
    unsafe {
      drop(Box::from_raw(ptr));
    }
  }
}
