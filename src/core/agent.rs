use std::{collections::HashMap, fs, process::Command, time::Duration};
use tokio::time::sleep;

use crate::core::{automata::Automata, bettercap::Bettercap, config::Config, log::LOGGER, utils::iface_channels};

const WIFI_RECON: &str = "wifi.recon";

pub struct Agent {
  config: Config,
  bettercap: Bettercap,
  automata: Automata,
  pub started_at: std::time::SystemTime,
  pub current_channel: Option<u8>,
  pub total_aps: u32,
  pub aps_on_channel: u32,
  pub supported_channels: Vec<u8>,
  pub peers: Vec<Peer>,

  pub access_points: Vec<AccessPoint>,
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
  pub stations: Vec<String>,
  pub clients: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Peer {
  pub mac: String,
  pub last_channel: u8,
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
            bettercap: Bettercap::default(),
            config: Config::default(),
            automata: Automata::default(),
            started_at: std::time::SystemTime::now(),
            current_channel: None,
            total_aps: 0,
            aps_on_channel: 0,
            supported_channels: vec![1, 6, 11],
            access_points: Vec::new(),
            peers: Vec::new(),
            last_pwned: None,
            history: Vec::new(),
            handshakes: HashMap::new(),
            mode: "auto".into(),
        }
    }
    
}

impl Agent {
    pub fn new(config: Config) -> Self {
        let bettercap = Bettercap::new(&config);
        let mut agent = Self  {
            bettercap,
            config: config.clone(),
            automata: Automata::new(config),
            started_at: std::time::SystemTime::now(),
            current_channel: None,
            total_aps: 0,
            aps_on_channel: 0,
            supported_channels: vec![1, 6, 11],
            access_points: Vec::new(),
            peers: Vec::new(),
            last_pwned: None,
            history: Vec::new(),
            handshakes: HashMap::new(),
            mode: "auto".into(),
        };
        agent.initialize();
        agent
    }

    fn initialize(&mut self) {
        LOGGER.log_info("Agent", "Initializing agent...");

        let handshakes_path = self.config.main.handshakes_path.clone();

        if !fs::metadata(&handshakes_path).is_ok() {
            if let Err(e) = fs::create_dir_all(&handshakes_path) {
                LOGGER.log_error("Agent", &format!("Failed to create handshakes dir: {}", e));
                return; // or propagate the error instead of ignoring
            }
        } else {
            LOGGER.log_info("Agent", &format!("Handshakes directory already exists: {}", handshakes_path));
        }

        LOGGER.log_info("Pwnagotchi",
          &format!("{}@{} (v{})",
            self.config.main.name,
            self.config.main.interface,
            env!("CARGO_PKG_VERSION")))
    }

    pub async fn setup_events(&mut self) {
        for event in self.config.bettercap.silence.iter() {
            self.bettercap.run(&["set", "events.ignore", event]).await.ok();
        }
    }

    async fn reset_wifi_settings(&mut self) {
        let interface = self.config.main.interface.clone();
        self.bettercap.run(&["set", "wifi.interface", &interface]).await.ok();
        self.bettercap.run(&["set", "wifi.ap.ttl", &format!("{}", self.config.personality.ap_ttl)]).await.ok();
        self.bettercap.run(&["set", "wifi.sta.ttl", &format!("{}", self.config.personality.sta_ttl)]).await.ok();
        self.bettercap.run(&["set", "wifi.rssi.min", &format!("{}", self.config.personality.min_rssi)]).await.ok();
        self.bettercap.run(&["set", "wifi.handshakes.file", &self.config.main.handshakes_path]).await.ok();
        self.bettercap.run(&["set", "wifi.handshakes.aggregate", false.to_string().as_str()]).await.ok();
    }

    pub async fn start_monitor_mode(&mut self) {
        LOGGER.log_info("Agent", "Starting monitor mode...");
        let interface = &self.config.main.interface;
        let mon_start_cmd = &self.config.main.mon_start_cmd;
        let restart = !&self.config.main.no_restart;
        let mut is_starting = false;
        let mut has_iface = false;

        while !has_iface {
            let session = self.bettercap.session(None).await.ok();

            if let Some(session) = session {
                for iface in session.interfaces {
                    if iface.name == *interface {
                        LOGGER.log_info("Agent", &format!("Found Monitor interface: {}", interface));
                        has_iface = true;
                        break;
                    }
                }

                if !has_iface {
                    if !is_starting && !mon_start_cmd.trim().is_empty() {
                        LOGGER.log_info("Agent", &format!("Starting monitor mode on {}", interface));

                        match Command::new("sh").arg("-c").arg(mon_start_cmd).output() {
                            Ok(output) if output.status.success() => {
                                LOGGER.log_info("Agent", "Monitor mode command executed successfully");
                            }
                            Ok(output) => {
                                LOGGER.log_error("Agent", &format!("Monitor mode failed: {:?}", output));
                            }
                            Err(e) => {
                                LOGGER.log_error("Agent", &format!("Failed to run monitor mode command: {}", e));
                            }
                        }
                        is_starting = true;
                    } else {
                        LOGGER.log_info("Agent", &format!("Monitor interface {} not found, waiting...", interface));
                        sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        }

        LOGGER.log_info("Agent", &format!("Supported channels: {:?}", self.supported_channels()));

        self.reset_wifi_settings().await;

        let wifi_running = self.is_module_running("wifi").await;

        if wifi_running && restart {
            LOGGER.log_debug("Agent", "Restarting WiFi module...");
            self.restart_module(WIFI_RECON).await;
            self.bettercap.run(&["wifi.clear"]).await.ok();
        } else if !wifi_running {
            LOGGER.log_info("Agent", "Starting WiFi module...");
            self.start_module(WIFI_RECON).await;
        }

        // Advertising logic here

    }

    async fn wait_for_bettercap(&mut self) {
        loop {
            match self.bettercap.session(None).await {
                Ok(_session) => {
                    return;
                }
                Err(_) => {
                    LOGGER.log_info("Agent", "Waiting for Bettercap...");
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }
        }
    }

    pub async fn start(&mut self) {
        self.wait_for_bettercap().await;
        self.setup_events().await;
        self.start_monitor_mode().await;
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

    pub async fn recon(mut self) {
        let mut recon_time = self.config.personality.recon_time;
        let max_inactive = self.config.personality.max_inactive_scale;
        let recon_multiplier = self.config.personality.recon_inactive_multiplier;
        let channels = &self.config.personality.channels;

        if self.automata.epoch.inactive_for >= max_inactive {
            recon_time *= recon_multiplier;
        }

        if channels.is_empty() {
            self.current_channel = None;
            LOGGER.log_debug("RECON", "No channels available for recon.");
            self.bettercap.run(&["wifi.recon.channel", "clear"]).await.ok();
        } else {
            let channel_str = channels.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(",");
            self.bettercap.run(&["wifi.recon.channel", "set", &channel_str]).await.ok();
        }

        LOGGER.log_debug("RECON", &format!("Recon time set to {} seconds", recon_time));
        // WAIT HERE

        self.automata.wait_for(recon_time, Some(false));
    }

    pub fn set_access_points(&mut self, aps: Vec<AccessPoint>) -> Vec<AccessPoint> {
        self.access_points = aps;
        self.automata.epoch.observe(self.access_points.clone(), self.peers.clone());
        self.access_points.clone()
    }

    pub fn get_access_points(&mut self) -> Vec<AccessPoint> {
        let whitelist = self.config.main.whitelist.clone();
        let aps = self.access_points.clone();
        let mut filtered_aps: Vec<AccessPoint> = aps
            .iter()
            .filter(|ap| whitelist.contains(&ap.bssid))
            .filter(|ap| whitelist.contains(&ap.ssid))
            .cloned()
            .collect();

        filtered_aps.sort_by_key(|ap| ap.signal);
        self.set_access_points(filtered_aps.clone());
        filtered_aps
    }

    pub fn get_total_aps(&self) -> usize {
        self.access_points.len()
    }

    pub fn get_aps_on_channel(&self, channel: u8) -> Vec<AccessPoint> {
        self.access_points
            .iter()
            .filter(|ap| ap.channel == channel)
            .cloned()
            .collect()
    }

    pub fn supported_channels(&self) -> Vec<u8> {
        iface_channels(&self.config.main.interface)
    }

    pub async fn is_module_running(&self, module: &str) -> bool {
        let session_result = self.bettercap.session(None).await;

        if let Ok(session) = session_result {
            if let Some(modules) = session.modules.get(module) {
                if let Some(is_running) = modules.get("running").and_then(|v| v.as_bool()) {
                    if is_running {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub async fn restart_module(&self, module: &str) {
        self.bettercap.run(&[module, "off"]).await.ok();
        self.bettercap.run(&[module, "on"]).await.ok();
    }

    pub async fn start_module(&self, module: &str) {
        self.bettercap.run(&[module, "on"]).await.ok();
    }
}
