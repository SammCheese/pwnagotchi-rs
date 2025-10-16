use std::{collections::HashMap, error::Error, sync::Arc};

use pwnagotchi_shared::{config::config, logger::LOGGER, traits::general::CoreModules};

use crate::{
  loaders::rust::RustPluginLoader,
  managers::hook_manager::HookManager,
  traits::{loaders::PluginLoader, plugins::Plugin},
};

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
enum PluginState {
  Registered,
  Initialized,
  Failed,
  Disabled,
}

pub struct PluginEntry {
  plugin: Box<dyn Plugin>,
  #[allow(dead_code)]
  id: String,
  state: PluginState,
  error: Option<String>,
}

pub struct PluginManager {
  plugins: HashMap<String, PluginEntry>,
  loaders: Vec<Box<dyn PluginLoader>>,
  core_modules: Option<Arc<CoreModules>>,
  hook_manager: HookManager,
}

impl Default for PluginManager {
  fn default() -> Self {
    Self::new()
  }
}

impl PluginManager {
  pub fn new() -> Self {
    PluginManager {
      plugins: HashMap::new(),
      hook_manager: HookManager::new(),
      core_modules: None,
      loaders: Vec::new(),
    }
  }

  pub fn init(&mut self) {
    println!("Initializing Plugin Manager...");
    self.loaders.push(Box::new(RustPluginLoader::new()));
  }

  pub fn reload_plugin(&mut self, _name: &str) -> Result<(), Box<dyn Error>> {
    Ok(())
  }

  pub fn load_plugins(&mut self) {
    let mut to_register = Vec::new();

    for loader in &mut self.loaders {
      // Command Loader to Load Plugins
      loader.load_plugins();

      // Hand Plugins over to Manager
      let plugins = loader.take_plugins();

      for plugin in plugins {
        if self.plugins.contains_key(plugin.info().name) {
          eprintln!("Plugin '{}' is already loaded, skipping.", plugin.info().name);
          continue;
        }
        to_register.push((plugin.info().name.to_string(), plugin));
      }
    }

    for (name, plugin) in to_register {
      // Unless directly set as disabled, enable
      let state = if let Some(pc) = config().plugins.get(&name)
        && !pc.enabled
      {
        PluginState::Disabled
      } else {
        PluginState::Registered
      };
      self.register_plugin(name, plugin, Some(state));
    }

    println!("Total plugins loaded: {}", self.plugins.len());
  }

  pub fn toggle_plugin(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
    if let Some(entry) = self.plugins.get(name) {
      if entry.state == PluginState::Initialized {
        self.disable_plugin(name)
      } else {
        self.enable_plugin(name)
      }
    } else {
      Err(format!("Plugin '{}' not found", name).into())
    }
  }

  pub fn set_coremodules(&mut self, core: Arc<CoreModules>) {
    self.core_modules = Some(core);
  }

  pub fn initialize_plugins(&mut self) {
    let plugin_names: Vec<String> = self
      .plugins
      .iter()
      .filter(|(_, entry)| entry.state == PluginState::Registered)
      .map(|(name, _)| name.clone())
      .collect();

    let mut failed = Vec::new();

    for name in plugin_names {
      if let Some(entry) = self.plugins.get_mut(&name) {
        if self.core_modules.is_none() {
          LOGGER.log_error("PLUGIN", "CoreModules not set, cannot initialize plugins.");
          eprintln!("CoreModules not set, cannot initialize plugins.");
          entry.state = PluginState::Failed;
          entry.error = Some("CoreModules not set".to_string());
          failed.push(name.clone());
          continue;
        }

        let core = Arc::clone(self.core_modules.as_ref().unwrap());
        let mut scoped_api = self.hook_manager.scope(&name);

        match entry.plugin.on_load(&mut scoped_api, core) {
          Ok(_) => {
            entry.state = PluginState::Initialized;
            println!("Plugin '{}' initialized successfully.", name);
          }
          Err(e) => {
            if let Err(hook_err) = self.hook_manager.unregister_plugin(&name) {
              LOGGER.log_error(
              "PLUGIN",
              &format!("Failed to unregister hooks for plugin '{name}': {hook_err}! EXPECT UNPREDICTABLE BEHAVIOR!"),
            );

              eprintln!(
                "Failed to unregister hooks for plugin '{name}': {hook_err}! EXPECT UNPREDICTABLE BEHAVIOR!"
              );
            }
            LOGGER.log_error("PLUGIN", &format!("Failed to start plugin '{name}': {e}"));
            eprintln!("Failed to start plugin '{name}': {e}");
            entry.state = PluginState::Failed;
            entry.error = Some(e.to_string());
            failed.push(name.clone());
          }
        }
      }
    }
  }

  fn register_plugin(&mut self, name: String, plugin: Box<dyn Plugin>, state: Option<PluginState>) {
    let state = state.unwrap_or(PluginState::Disabled);
    self.plugins.insert(
      name,
      PluginEntry {
        plugin,
        id: uuid::Uuid::new_v4().to_string(),
        state,
        error: None,
      },
    );
  }

  fn unregister_plugin(&mut self, name: &str) -> Result<Option<Box<dyn Plugin>>, Box<dyn Error>> {
    self.hook_manager.unregister_plugin(name)?;
    if let Some(mut entry) = self.plugins.remove(name) {
      entry.plugin.on_unload()?;
      Ok(Some(entry.plugin))
    } else {
      Err(format!("Plugin '{}' not found", name).into())
    }
  }

  pub fn enable_plugin(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
    if let Some(entry) = self.plugins.get_mut(name) {
      if self.core_modules.is_none() {
        LOGGER.log_error("PLUGIN", "CoreModules not set, cannot initialize plugins.");
        eprintln!("CoreModules not set, cannot initialize plugins.");
        entry.state = PluginState::Failed;
        entry.error = Some("CoreModules not set".to_string());
        return Err("CoreModules not set".into());
      }

      let mut scoped_api = self.hook_manager.scope(name);
      let core = Arc::clone(self.core_modules.as_ref().unwrap());
      match entry.plugin.on_load(&mut scoped_api, core) {
        Ok(_) => {
          entry.state = PluginState::Initialized;
          Ok(())
        }
        Err(e) => {
          let _ = self.hook_manager.unregister_plugin(name);
          entry.state = PluginState::Failed;
          entry.error = Some(e.to_string());
          LOGGER.log_error("PLUGIN", &format!("Failed to start plugin '{}': {}", name, e));
          Err(format!("Failed to start plugin '{}': {}", name, e).into())
        }
      }
    } else {
      Err(format!("Plugin '{}' not found", name).into())
    }
  }

  pub fn disable_plugin(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
    if let Some(entry) = self.plugins.get_mut(name) {
      entry.plugin.on_unload()?;
      self.hook_manager.unregister_plugin(name)?;
      entry.state = PluginState::Disabled;
      Ok(())
    } else {
      Err(format!("Plugin '{}' not found", name).into())
    }
  }

  pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
    self.plugins.get(name).map(|boxed| boxed.plugin.as_ref())
  }

  pub fn shutdown_all(&mut self) -> Result<(), Box<dyn Error>> {
    for name in self.plugins.keys().cloned().collect::<Vec<_>>() {
      let _ = self.unregister_plugin(&name);
    }
    Ok(())
  }
}
