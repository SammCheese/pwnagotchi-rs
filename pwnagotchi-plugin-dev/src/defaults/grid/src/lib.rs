use std::{
  collections::HashSet,
  error::Error,
  path::PathBuf,
  sync::{Arc, Mutex},
  thread,
  time::Duration,
};

use pwnagotchi_plugins::traits::{
  events::EventHandler,
  plugins::{Plugin, PluginAPI, PluginInfo},
};
use pwnagotchi_shared::{
  logger::LOGGER, sessions::session_stats::SessionStats, traits::general::CoreModules,
  types::events::EventPayload,
};
use regex::Regex;

#[derive(Default)]
pub struct Grid {
  reported: Arc<Mutex<HashSet<String>>>,
  is_locked: Arc<Mutex<bool>>,
}

impl Grid {
  pub fn new() -> Self {
    Self {
      reported: Arc::new(Mutex::new(HashSet::new())),
      is_locked: Arc::new(Mutex::new(false)),
    }
  }

  #[allow(unused)]
  fn parse_pcap(&self, filename: &str) -> Option<(String, String)> {
    LOGGER.log_debug("Grid", &format!("Parsing {}", filename));

    let base = filename.trim_end_matches(".pcap");
    let (essid, mut bssid) = if base.contains('_') {
      let parts: Vec<&str> = base.splitn(2, '_').collect();
      (parts[0].to_string(), parts[1].to_string())
    } else {
      ("".into(), base.to_string())
    };

    let mac_re = Regex::new(r"^[0-9a-fA-F]{12}$").unwrap();
    if !mac_re.is_match(&bssid) {
      return None;
    }

    // Format 12-digit hex MAC into colon form
    let chars: Vec<char> = bssid.chars().collect();
    bssid = chars
      .chunks(2)
      .map(|pair| pair.iter().collect::<String>())
      .collect::<Vec<String>>()
      .join(":");

    Some((essid, bssid))
  }

  fn check_handshakes(&mut self, core: &Arc<CoreModules>, bettercap_path: &str) {
    let mut reported = self.reported.lock().unwrap();
    let pcap_dir = PathBuf::from(bettercap_path);
    let Ok(entries) = std::fs::read_dir(pcap_dir) else {
      return;
    };

    for entry in entries.flatten() {
      let path = entry.path();
      if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if !name.ends_with(".pcap") {
          continue;
        }
        let net_id = name.trim_end_matches(".pcap");
        if reported.contains(net_id) {
          continue;
        }
        if let Some((essid, bssid)) = self.parse_pcap(name) {
          LOGGER.log_info(
            "Grid",
            &format!("Reporting new handshake ESSID='{}', BSSID='{}'", essid, bssid),
          );
          core.grid.report_ap(&essid, &bssid);
          reported.insert(net_id.to_string());
          thread::sleep(Duration::from_millis(1500));
        }
      }
    }
  }

  fn handle_internet_available(&mut self, plugin_api: &PluginAPI) -> EventHandler {
    let grid_ref = Arc::clone(&plugin_api.core_modules.grid);
    let reported = Arc::clone(&self.reported);
    let lock_flag = Arc::clone(&self.is_locked);
    let core = Arc::clone(&plugin_api.core_modules);
    let bettercap_path = plugin_api.config.bettercap.handshakes.clone();

    let handler: EventHandler = Arc::new(move |payload: &EventPayload| {
      let mut lock = lock_flag.lock().unwrap();
      if *lock {
        return Ok(());
      }
      *lock = true;

      let stats = payload.deserialize::<SessionStats>()?;
      grid_ref.update_data(&stats);

      drop(lock);

      let mut tmp = Grid {
        reported: Arc::clone(&reported),
        is_locked: Arc::clone(&lock_flag),
      };
      tmp.check_handshakes(&core, &bettercap_path);

      Ok(())
    });
    handler
  }
}

impl Plugin for Grid {
  fn info(&self) -> &PluginInfo {
    &PluginInfo {
      name: "Grid",
      version: "1.0.0",
      author: "SammCheese",
      description: "Signal the Identity, Pwned Networks and Networks to opwngrid.xyz",
      license: "MIT",
    }
  }

  fn on_load(&mut self, plugin_api: PluginAPI) -> Result<(), Box<dyn Error + 'static>> {
    plugin_api
      .event_api
      .register_listener("internet_available", self.handle_internet_available(&plugin_api))?;
    Ok(())
  }

  fn on_unload(&mut self) -> Result<(), Box<dyn Error + 'static>> {
    Ok(())
  }
}

#[allow(improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn _plugin_create() -> *mut dyn Plugin {
  let plugin: Box<dyn Plugin> = Box::new(Grid::new());
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
