use std::{collections::HashMap, fs};

use tungstenite::handshake;

use crate::core::{bettercap::Bettercap, config::{self, Config, CONFIG}, log::LOGGER};



pub struct Agent {
  bettercap: Option<Bettercap>,
  config: Option<Config>,
  pub started_at: std::time::SystemTime,
  pub current_channel: Option<u8>,
  pub total_aps: u32,
  pub aps_on_channel: u32,
  pub supported_channels: Vec<u8>,
  
  pub access_points: HashMap<String, AccessPoint>,
  pub last_pwned: Option<String>,
  pub history: Vec<String>,
  pub handshakes: HashMap<String, Handshake>,
  pub mode: String,
}

#[derive(Debug, Clone)]
pub struct AccessPoint {
  pub ssid: String,
  pub bssid: String,
  pub channel: u8,
  pub signal: i32,
  pub pwned: bool,
}

#[derive(Debug, Clone)]
pub struct Handshake {
  pub bssid: String,
  pub client_mac: String,
  pub timestamp: std::time::SystemTime,
  pub file_path: String,
}

impl Default for Agent {
    fn default() -> Self {
        Agent {
            bettercap: None,
            config: None,
            started_at: std::time::SystemTime::now(),
            current_channel: None,
            total_aps: 0,
            aps_on_channel: 0,
            supported_channels: vec![1, 6, 11],
            access_points: HashMap::new(),
            last_pwned: None,
            history: Vec::new(),
            handshakes: HashMap::new(),
            mode: "auto".into(),
        }
    }
    
}

impl Agent {
    pub fn new() -> Self {
        let mut agent = Agent  {
            config: Some(CONFIG.clone()),
            ..Agent::default()
        };
        agent.initialize();
        agent
    }

    fn initialize(&mut self) {
        LOGGER.log_info("Agent", "Initializing agent...");

        self.bettercap = Some(Bettercap::new(&self.config.as_ref().unwrap()));
        
        let handshakes_path = self.config.as_ref().unwrap().main.handshakes_path.clone();

        if !fs::metadata(&handshakes_path).is_ok() {
            fs::create_dir_all(&handshakes_path)
                .unwrap_or_else(|e| eprintln!("Failed to create handshakes directory: {}", e));
        } else {
            eprintln!("Handshakes directory already exists: {}", handshakes_path);
        }

        LOGGER.log_info("Pwnagotchi",
          &format!("{}@{} (v{})",
            self.config.as_ref().unwrap().main.name,
            self.config.as_ref().unwrap().main.wifi_interface,
            env!("CARGO_PKG_VERSION")))
    }



    pub fn reset(&mut self) {
        self.started_at = std::time::SystemTime::now();
        self.current_channel = None;
        self.total_aps = 0;
        self.aps_on_channel = 0;
        self.access_points.clear();
        self.last_pwned = None;
        self.history.clear();
        self.handshakes.clear();
    }
}
