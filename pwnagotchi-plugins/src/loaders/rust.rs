use std::{
  error::Error,
  panic::{AssertUnwindSafe, catch_unwind},
  sync::Arc,
};

use libloading::{Library, Symbol};
use pwnagotchi_shared::{config::config, logger::LOGGER, traits::general::CoreModules};
use uuid::Uuid;

use crate::{
  managers::{
    event_manager::{DynamicEventAPI, EventManager},
    hook_manager::{DynamicHookAPI, HookManager},
    plugin_manager::{PluginEntry, PluginState},
  },
  traits::{events::DynamicEventAPITrait, loaders::PluginLoader, plugins::Plugin},
};

pub struct RustPluginLoader {
  #[allow(clippy::vec_box)]
  plugins: Vec<Box<PluginEntry>>,
  libraries: Vec<Library>,
  hook_manager: Arc<HookManager>,
  event_manager: Arc<EventManager>,
  core: Option<Arc<CoreModules>>,
}

impl RustPluginLoader {
  pub fn new(hook_manager: Arc<HookManager>, event_manager: Arc<EventManager>) -> Self {
    Self {
      plugins: Vec::new(),
      libraries: Vec::new(),
      hook_manager,
      event_manager,
      core: None,
    }
  }
}

impl PluginLoader for RustPluginLoader {
  fn name(&self) -> &str {
    "RustPluginLoader"
  }

  fn extension(&self) -> &str {
    #[cfg(target_os = "windows")]
    return "dll";
    #[cfg(target_os = "linux")]
    return "so";
    #[cfg(target_os = "macos")]
    return "dylib";
  }

  fn load_plugins(&mut self) -> Result<(), Box<dyn Error>> {
    let plugin_dir = config().main.plugins_path.clone();

    if let Some(plugin_dir) = plugin_dir {
      let plugin_dir = plugin_dir.to_string();
      std::fs::create_dir_all(&plugin_dir).unwrap();

      for entry in std::fs::read_dir(plugin_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some(self.extension()) {
          println!("Found Rust plugin: {:?}", path.file_name());
          unsafe {
            if let Err(e) = self.load_plugin_from_path(&path) {
              println!("Failed to load plugin from {:?}: {}", path.file_name(), e);
            }
          }
        }
      }
    }
    Ok(())
  }

  fn unload_all(&mut self) -> Result<(), Box<dyn Error>> {
    for entry in &mut self.plugins {
      let plugin = &mut entry.plugin;
      match catch_unwind(AssertUnwindSafe(|| plugin.on_unload())) {
        Ok(Ok(())) => {
          entry.state = PluginState::Unloaded;
          entry.error = None;
          self.hook_manager.unregister_plugin(plugin.info().name).ok();
          self.event_manager.unregister_plugin(plugin.info().name).ok();
          LOGGER
            .log_info("PLUGIN", &format!("Successfully unloaded plugin '{}'", plugin.info().name));
          println!("Successfully unloaded plugin '{}'", plugin.info().name);
        }
        Ok(Err(e)) => {
          entry.state = PluginState::Failed;
          entry.error = Some(e.to_string());
          self.hook_manager.unregister_plugin(plugin.info().name).ok();
          self.event_manager.unregister_plugin(plugin.info().name).ok();
          println!("Failed to unload plugin '{}': {}", plugin.info().name, e);
        }
        Err(_) => {
          entry.state = PluginState::Failed;
          self.event_manager.unregister_plugin(plugin.info().name).ok();
          println!("Plugin '{}' panicked during unloading", plugin.info().name);
        }
      }
    }
    self.plugins.clear();
    self.libraries.clear();
    Ok(())
  }

  fn init_all(&mut self, core: Arc<CoreModules>) -> Result<(), Box<dyn Error>> {
    // lets save this for later
    self.core = Some(Arc::clone(&core));
    let disabled_plugins: Vec<String> = config()
      .plugins
      .iter()
      .filter(|s| !s.1.enabled)
      .map(|s| s.0.to_string())
      .collect();

    for entry in &mut self.plugins {
      if disabled_plugins.contains(&entry.plugin.info().name.to_string()) {
        println!("Skipping disabled plugin '{}'", entry.plugin.info().name);
        continue;
      }

      let plugin = &mut entry.plugin;
      let core = Arc::clone(&core);
      let mut hook_api = self.hook_manager.scope(plugin.info().name);
      let mut event_api = self.event_manager.scope(plugin.info().name);

      // Safely catch panics from plugins
      match catch_unwind(AssertUnwindSafe(|| plugin.on_load(&mut hook_api, &mut event_api, core))) {
        Ok(Ok(())) => {
          LOGGER.log_info(
            "PLUGIN",
            &format!("Successfully initialized plugin '{}'", plugin.info().name),
          );
          println!("Successfully initialized plugin '{}'", plugin.info().name);
          entry.state = PluginState::Initialized;
          entry.error = None;
        }
        Ok(Err(e)) => {
          entry.state = PluginState::Failed;
          entry.error = Some(e.to_string());
          let _ = event_api.cleanup();
          println!("Failed to initialize plugin '{}': {}", plugin.info().name, e);
        }
        Err(_) => {
          let _ = event_api.cleanup();
          println!("Plugin '{}' panicked during initialization", plugin.info().name);
        }
      }
    }
    Ok(())
  }

  fn get_plugins(&self) -> &Vec<Box<PluginEntry>> {
    &self.plugins
  }

  fn enable_plugin(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
    for entry in &mut self.plugins {
      let plugin = &mut entry.plugin;
      if plugin.info().name != name {
        continue;
      }

      let hook_api = &mut DynamicHookAPI {
        manager: &Arc::clone(&self.hook_manager),
        plugin_name: plugin.info().name.to_string(),
        registered_hooks: Vec::new(),
      };

      let event_api = &mut DynamicEventAPI {
        manager: &self.event_manager,
        plugin_name: plugin.info().name.to_string(),
        registered_listeners: Vec::new(),
      };

      match catch_unwind(AssertUnwindSafe(|| {
        let core = self.core.as_ref().ok_or("CoreModules not initialized")?;
        plugin.on_load(hook_api, event_api, Arc::clone(core))
      })) {
        Ok(Ok(())) => {
          entry.state = PluginState::Initialized;
          entry.error = None;
          return Ok(());
        }
        Ok(Err(e)) => {
          entry.state = PluginState::Failed;
          let _ = event_api.cleanup();
          entry.error = Some(e.to_string());
          return Err(format!("Failed to enable plugin '{}': {}", name, e).into());
        }
        Err(_) => {
          entry.state = PluginState::Failed;
          let _ = event_api.cleanup();
          return Err(format!("Plugin '{}' panicked during enabling", name).into());
        }
      }
    }
    Err(format!("Plugin '{}' not found", name).into())
  }

  fn disable_plugin(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
    for entry in &mut self.plugins {
      let plugin = &mut entry.plugin;
      if plugin.info().name != name {
        continue;
      }
      match catch_unwind(AssertUnwindSafe(|| plugin.on_unload())) {
        Ok(Ok(())) => {
          self.hook_manager.unregister_plugin(plugin.info().name).ok();
          self.event_manager.unregister_plugin(plugin.info().name).ok();

          entry.state = PluginState::Unloaded;
          return Ok(());
        }
        Ok(Err(e)) => {
          entry.state = PluginState::Failed;
          self.event_manager.unregister_plugin(plugin.info().name).ok();
          return Err(format!("Failed to disable plugin '{}': {}", name, e).into());
        }
        Err(_) => {
          entry.state = PluginState::Failed;
          self.event_manager.unregister_plugin(plugin.info().name).ok();
          return Err(format!("Plugin '{}' panicked during disabling", name).into());
        }
      }
    }
    Err(format!("Plugin '{}' not found", name).into())
  }
}

impl RustPluginLoader {
  unsafe fn load_plugin_from_path(&mut self, path: &std::path::Path) -> Result<(), Box<dyn Error>> {
    unsafe {
      let lib = Library::new(path)?;

      let constructor: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> =
        lib.get(b"_plugin_create")?;

      let plugin_ptr = constructor();
      let plugin = Box::from_raw(plugin_ptr);

      let entry = PluginEntry {
        plugin,
        id: Uuid::new_v4().to_string(),
        state: PluginState::Registered,
        error: None,
      };

      self.plugins.push(Box::new(entry));
      self.libraries.push(lib);

      Ok(())
    }
  }
}
