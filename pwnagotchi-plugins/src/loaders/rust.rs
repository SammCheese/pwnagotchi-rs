use std::error::Error;

use libloading::{Library, Symbol};
use pwnagotchi_shared::config::config;

use crate::traits::{loaders::PluginLoader, plugins::Plugin};

pub struct RustPluginLoader {
  plugins: Vec<Box<dyn Plugin>>,
  libraries: Vec<Library>,
}

impl Default for RustPluginLoader {
  fn default() -> Self {
    Self::new()
  }
}

impl RustPluginLoader {
  pub fn new() -> Self {
    Self {
      plugins: Vec::new(),
      libraries: Vec::new(),
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

  fn load_plugins(&mut self) {
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
  }

  // Used to take the Plugins out of the loader and into the PluginManager
  fn take_plugins(&mut self) -> Vec<Box<dyn Plugin>> {
    std::mem::take(&mut self.plugins)
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

      self.plugins.push(plugin);
      self.libraries.push(lib);

      Ok(())
    }
  }
}
