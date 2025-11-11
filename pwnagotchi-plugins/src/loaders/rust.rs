use std::{
  error::Error,
  panic::{AssertUnwindSafe, catch_unwind},
  sync::Arc,
  thread,
};

use libloading::{Library, Symbol};
use pwnagotchi_shared::{config::config_read, logger::LOGGER, traits::general::CoreModules};
use uuid::Uuid;

use crate::{
  managers::{
    event_manager::{DynamicEventAPI, EventManager},
    hook_manager::{DynamicHookAPI, HookManager},
    plugin_manager::{PluginEntry, PluginState},
  },
  traits::{
    loaders::PluginLoader,
    plugins::{Plugin, PluginAPI},
  },
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
    let plugin_dir = config_read().main.plugins_path.clone();

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
      let plugin_name = entry.plugin.info().name.to_string();
      let plugin = &mut *entry.plugin;

      let result = thread::scope(|s| {
        s.spawn(|| {
          catch_unwind(AssertUnwindSafe(|| plugin.on_unload()))
            .map(|res| res.map_err(|e| e.to_string()))
        })
        .join()
      });

      match result {
        Ok(Ok(Ok(()))) => {
          entry.state = PluginState::Unloaded;
          entry.error = None;
          self.hook_manager.unregister_plugin(&plugin_name).ok();
          self.event_manager.unregister_plugin(&plugin_name).ok();
          LOGGER.log_info("PLUGIN", &format!("Successfully unloaded plugin '{}'", plugin_name));
          println!("Successfully unloaded plugin '{}'", plugin_name);
        }
        Ok(Ok(Err(e))) => {
          entry.state = PluginState::Failed;
          entry.error = Some(e.clone());
          self.hook_manager.unregister_plugin(&plugin_name).ok();
          self.event_manager.unregister_plugin(&plugin_name).ok();
          println!("Failed to unload plugin '{}': {}", plugin_name, e);
        }
        Ok(Err(_)) => {
          entry.state = PluginState::Failed;
          self.event_manager.unregister_plugin(&plugin_name).ok();
          println!("Plugin '{}' panicked during unloading", plugin_name);
        }
        Err(e) => {
          entry.state = PluginState::Failed;
          println!("Thread panicked for plugin '{}': {:?}", plugin_name, e);
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
    let disabled_plugins: Vec<String> = config_read()
      .plugins
      .iter()
      .filter(|s| !s.1.enabled)
      .map(|s| s.0.to_string())
      .collect();

    for entry in &mut self.plugins {
      let plugin_name = entry.plugin.info().name.to_string();

      if disabled_plugins.contains(&plugin_name) {
        println!("Skipping disabled plugin '{}'", plugin_name);
        continue;
      }

      let logger = Arc::new(&LOGGER);

      let plugin_api = PluginAPI {
        hook_api: &mut DynamicHookAPI {
          manager: &Arc::clone(&self.hook_manager),
          plugin_name: plugin_name.clone(),
          registered_hooks: Vec::new(),
        },
        event_api: &mut DynamicEventAPI {
          manager: &self.event_manager,
          plugin_name: plugin_name.clone(),
          registered_listeners: Vec::new(),
        },
        core_modules: Arc::clone(&core),
        logger,
        config: &config_read(),
      };

      let plugin = &mut *entry.plugin;

      let result = thread::scope(|s| {
        s.spawn(|| {
          catch_unwind(AssertUnwindSafe(|| plugin.on_load(plugin_api)))
            .map(|res| res.map_err(|e| e.to_string()))
        })
        .join()
      });

      match result {
        Ok(Ok(Ok(()))) => {
          LOGGER.log_info("PLUGIN", &format!("Successfully initialized plugin '{}'", plugin_name));
          println!("Successfully initialized plugin '{}'", plugin_name);
          entry.state = PluginState::Initialized;
          entry.error = None;
        }
        Ok(Ok(Err(e))) => {
          entry.state = PluginState::Failed;
          entry.error = Some(e.clone());
          println!("Failed to initialize plugin '{}': {}", plugin_name, e);
        }
        Ok(Err(_)) => {
          entry.state = PluginState::Failed;
          println!("Plugin '{}' panicked during initialization", plugin_name);
        }
        Err(e) => {
          entry.state = PluginState::Failed;
          println!("Thread panicked for plugin '{}': {:?}", plugin_name, e);
        }
      }
    }
    Ok(())
  }

  fn get_plugins(&self) -> &Vec<Box<PluginEntry>> {
    &self.plugins
  }

  fn enable_plugin(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
    let entry = self.plugins.iter_mut().find(|e| e.plugin.info().name == name);

    if entry.is_none() {
      return Err(format!("Plugin '{}' not found", name).into());
    }

    let entry = entry.unwrap();
    let plugin = &mut *entry.plugin;

    let logger = Arc::new(&LOGGER);
    let plugin_name = plugin.info().name.to_string();

    let plugin_api = PluginAPI {
      hook_api: &mut DynamicHookAPI {
        manager: &Arc::clone(&self.hook_manager),
        plugin_name: plugin_name.clone(),
        registered_hooks: Vec::new(),
      },
      event_api: &mut DynamicEventAPI {
        manager: &self.event_manager,
        plugin_name: plugin_name.clone(),
        registered_listeners: Vec::new(),
      },
      core_modules: self.core.as_ref().ok_or("CoreModules not initialized")?.clone(),
      logger,
      config: &config_read(),
    };

    let plugin = &mut *entry.plugin;

    let result = thread::scope(|s| {
      s.spawn(|| {
        catch_unwind(AssertUnwindSafe(|| plugin.on_load(plugin_api)))
          .map(|res| res.map_err(|e| e.to_string()))
      })
      .join()
    });

    match result {
      Ok(Ok(Ok(()))) => {
        LOGGER.log_info("PLUGIN", &format!("Successfully initialized plugin '{}'", plugin_name));
        println!("Successfully initialized plugin '{}'", plugin_name);
        entry.state = PluginState::Initialized;
        entry.error = None;
        Ok(())
      }
      Ok(Ok(Err(e))) => {
        entry.state = PluginState::Failed;
        entry.error = Some(e.clone());
        Err(format!("Failed to enable plugin '{}': {}", name, e).into())
      }
      Ok(Err(_)) => {
        entry.state = PluginState::Failed;
        Err(format!("Plugin '{}' panicked during enabling", name).into())
      }
      Err(e) => {
        entry.state = PluginState::Failed;
        Err(format!("Thread panicked for plugin '{}': {:?}", name, e).into())
      }
    }
  }

  fn disable_plugin(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
    let entry = self.plugins.iter_mut().find(|e| e.plugin.info().name == name);

    if entry.is_none() {
      return Err(format!("Plugin '{}' not found", name).into());
    }

    let entry = entry.unwrap();
    let plugin = &mut *entry.plugin;

    let result = thread::scope(|s| {
      s.spawn(|| {
        catch_unwind(AssertUnwindSafe(|| plugin.on_unload()))
          .map(|res| res.map_err(|e| e.to_string()))
      })
      .join()
    });

    match result {
      Ok(Ok(Ok(()))) => {
        self.hook_manager.unregister_plugin(name).ok();
        self.event_manager.unregister_plugin(name).ok();
        entry.state = PluginState::Unloaded;
        LOGGER.log_info("PLUGIN", &format!("Successfully disabled plugin '{}'", name));
        Ok(())
      }
      Ok(Ok(Err(e))) => {
        entry.state = PluginState::Failed;
        self.event_manager.unregister_plugin(name).ok();
        Err(format!("Failed to disable plugin '{}': {}", name, e).into())
      }
      Ok(Err(_)) => {
        entry.state = PluginState::Failed;
        self.event_manager.unregister_plugin(name).ok();
        Err(format!("Plugin '{}' panicked during disabling", name).into())
      }
      Err(e) => {
        entry.state = PluginState::Failed;
        Err(format!("Thread panicked for plugin '{}': {:?}", name, e).into())
      }
    }
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
