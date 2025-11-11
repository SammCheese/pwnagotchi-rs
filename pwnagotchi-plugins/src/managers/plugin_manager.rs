use std::{
  error::Error,
  panic::{AssertUnwindSafe, catch_unwind},
  sync::Arc,
};

use pwnagotchi_shared::{
  config::config_write,
  logger::LOGGER,
  traits::{events::EventBus, general::CoreModules},
  types::events::EventPayload,
};

use crate::{
  loaders::rust::RustPluginLoader,
  managers::{event_manager::EventManager, hook_manager::HookManager},
  traits::{loaders::PluginLoader, plugins::Plugin},
};

pub fn safe_call<F>(plugin: &mut dyn Plugin, f: F) -> Result<(), Box<dyn Error>>
where
  F: FnOnce(&mut dyn Plugin) -> Result<(), Box<dyn Error>>,
{
  match catch_unwind(AssertUnwindSafe(|| f(plugin))) {
    Ok(Ok(())) => Ok(()),
    Ok(Err(e)) => Err(e),
    Err(_) => Err("Plugin panicked".into()),
  }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum PluginState {
  Registered,
  Initialized,
  Failed,
  Disabled,
  Unloaded,
}

pub struct PluginEntry {
  pub plugin: Box<dyn Plugin>,
  pub id: String,
  pub state: PluginState,
  pub error: Option<String>,
}

pub struct PluginManager {
  loaders: Vec<Box<dyn PluginLoader>>,
  core_modules: Option<Arc<CoreModules>>,
  hook_manager: Arc<HookManager>,
  event_manager: Arc<EventManager>,
}

impl Default for PluginManager {
  fn default() -> Self {
    Self::new()
  }
}

impl PluginManager {
  pub fn new() -> Self {
    PluginManager {
      hook_manager: Arc::new(HookManager::new()),
      event_manager: Arc::new(EventManager::new()),
      core_modules: None,
      loaders: Vec::new(),
    }
  }

  pub fn event_bus(&self) -> Arc<dyn EventBus> {
    Arc::clone(&self.event_manager) as Arc<dyn EventBus>
  }

  pub fn init(&mut self) {
    println!("Initializing Plugin Manager...");

    self.loaders.push(Box::new(RustPluginLoader::new(
      Arc::clone(&self.hook_manager),
      Arc::clone(&self.event_manager),
    )));
  }

  pub fn set_core_modules(&mut self, core: Arc<CoreModules>) {
    self.core_modules = Some(core);
  }

  pub fn reload_plugin(&mut self, _name: &str) -> Result<(), Box<dyn Error>> {
    Ok(())
  }

  pub fn load_plugins(&mut self) {
    for loader in &mut self.loaders {
      // Command Loader to Load Plugins
      let _ = loader.load_plugins();
    }

    let total_plugins: usize = self.loaders.iter().map(|l| l.get_plugins().len()).sum();
    println!("Loaded {} plugins.", total_plugins);
  }

  pub fn get_plugins(&self) -> Vec<&PluginEntry> {
    self
      .loaders
      .iter()
      .flat_map(|loader| loader.get_plugins().iter().map(|b| &**b))
      .collect()
  }

  pub fn initialize_plugins(&mut self) {
    if self.core_modules.is_none() {
      LOGGER.log_error("PLUGIN", "CoreModules not set, cannot initialize plugins.");
      eprintln!("CoreModules not set, cannot initialize plugins.");
      return;
    }

    for loader in self.loaders.iter_mut() {
      let _ = loader.init_all(Arc::clone(self.core_modules.as_ref().unwrap()));
    }

    let event = Arc::clone(&self.event_manager);
    tokio::spawn(async move {
      let _ = event.emit_payload_native("plugin::init", EventPayload::empty()).await;
    });
  }

  pub fn enable_plugin(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
    for loader in &mut self.loaders {
      if loader.get_plugins().iter().any(|p| p.plugin.info().name == name) {
        loader.enable_plugin(name)?;
        self.set_plugin_state(name, true);
        return Ok(());
      }
    }
    Err("Plugin not found".into())
  }

  pub fn disable_plugin(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
    for loader in &mut self.loaders {
      if loader.get_plugins().iter().any(|p| p.plugin.info().name == name) {
        loader.disable_plugin(name)?;
        self.set_plugin_state(name, false);
        return Ok(());
      }
    }
    Err("Plugin not found".into())
  }

  pub fn set_plugin_state(&self, name: &str, enabled: bool) {
    let mut config = config_write();
    let plugin_config = config.plugins.entry(name.to_string()).or_default();

    plugin_config.enabled = enabled;

    if plugin_config.config == Some(serde_json::Value::Null) {
      plugin_config.config = Some(serde_json::json!({}));
    }
  }

  pub fn shutdown_all(&mut self) -> Result<(), Box<dyn Error>> {
    for loader in &mut self.loaders {
      loader.unload_all()?;
    }
    Ok(())
  }
}
