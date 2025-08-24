use std::collections::HashSet;
use std::{ collections::HashMap, fs, process::Command, time::Duration };

use std::thread::sleep;
use tokio::sync::mpsc::{ self, Receiver, error::TryRecvError, Sender };
use serde::{ Deserialize, Serialize };

use crate::core::{
    automata::Automata,
    bettercap::{ Bettercap, BettercapSession },
    config::Config,
    identity::Identity,
    log::LOGGER,
    utils::{ self, iface_channels },
};

const WIFI_RECON: &str = "wifi.recon";

pub struct Agent {
    config: Config,
    bettercap: Bettercap,
    pub automata: Automata,
    identity: Identity,
    event_inbox: Option<tokio::sync::mpsc::Receiver<String>>,
    pub started_at: std::time::SystemTime,
    pub current_channel: Option<u8>,
    pub total_aps: u32,
    pub aps_on_channel: u32,
    pub supported_channels: Vec<u8>,
    pub peers: Vec<Peer>,

    pub access_points: Vec<AccessPoint>,
    pub last_pwned: Option<String>,
    pub history: HashMap<String, u32>,
    pub handshakes: HashMap<String, Handshake>,
    pub mode: String,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Station {
    pub mac: String,
    pub hostname: String,
    pub vendor: String,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct AccessPoint {
    pub channel: u8,
    pub signal: i32,
    pub mac: String,
    pub encryption: String,
    pub hostname: String,
    pub stations: Vec<Station>,
    pub clients: Vec<Station>,
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub mac: String,
    pub last_channel: u8,
}

#[derive(Debug, Clone)]
pub struct Handshake {
    pub mac: String,
    pub client_mac: String,
    pub timestamp: std::time::SystemTime,
    pub file_path: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
    pub event_type: String,
    pub data: HashMap<String, String>,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            bettercap: Bettercap::default(),
            config: Config::default(),
            automata: Automata::default(),
            identity: Identity::default(),
            event_inbox: None,
            started_at: std::time::SystemTime::now(),
            current_channel: None,
            total_aps: 0,
            aps_on_channel: 0,
            supported_channels: vec![1, 6, 11],
            access_points: Vec::new(),
            peers: Vec::new(),
            last_pwned: None,
            history: HashMap::new(),
            handshakes: HashMap::new(),
            mode: "auto".into(),
        }
    }
}

impl Agent {
    pub fn new(config: Config) -> Self {
        let bettercap = Bettercap::new(&config);
        let mut agent = Self {
            bettercap,
            config: config.clone(),
            automata: Automata::new(config),
            identity: Identity::new("./emulation/"),
            event_inbox: None,
            started_at: std::time::SystemTime::now(),
            current_channel: None,
            total_aps: 0,
            aps_on_channel: 0,
            supported_channels: vec![1, 6, 11],
            access_points: Vec::new(),
            peers: Vec::new(),
            last_pwned: None,
            history: HashMap::new(),
            handshakes: HashMap::new(),
            mode: "auto".into(),
        };
        agent.initialize();
        agent
    }

    fn initialize(&mut self) {
        LOGGER.log_info("Agent", "Initializing agent...");

        let handshakes_path = self.config.main.handshakes_path.clone();
        if
            fs::metadata(&handshakes_path).is_err() &&
            let Err(e) = fs::create_dir_all(&handshakes_path)
        {
            LOGGER.log_error("Agent", &format!("Failed to create handshakes dir: {e}"));
            return;
        }

        self.supported_channels = utils::iface_channels(&self.config.main.interface);
        self.identity.initialize();

        LOGGER.log_info(
            "Pwnagotchi",
            &format!(
                "{}@{} (v{})",
                self.config.main.name,
                self.identity.fingerprint(),
                env!("CARGO_PKG_VERSION")
            )
        );
    }

    #[allow(clippy::future_not_send)]
    pub async fn setup_events(&mut self) {
        LOGGER.log_info("Agent", "Setting up Bettercap events...");
        let bettercap = self.bettercap.clone();
        for event in &self.config.bettercap.silence {
            let ev = event.clone();
            let bc = bettercap.clone();
            if let Err(e) = bc.run(&["set", "events.ignore", ev.as_str()]).await {
                LOGGER.log_error("Agent", &format!("Failed to set events.ignore for {event}: {e}"));
            }
        }
    }

    #[allow(clippy::future_not_send)]
    pub async fn reset_wifi_settings(&self) {
        let interface = self.config.main.interface.clone();
        let bc = self.bettercap.clone();
        if let Err(e) = bc.run(&["set", "wifi.interface", &interface]).await {
            LOGGER.log_error("Agent", &format!("Failed to set wifi.interface: {e}"));
        }
        let bc = self.bettercap.clone();
        let ap_ttl = format!("{}", self.config.personality.ap_ttl);
        if let Err(e) = bc.run(&["set", "wifi.ap.ttl", &ap_ttl]).await {
            LOGGER.log_error("Agent", &format!("Failed to set wifi.ap.ttl: {e}"));
        }
        let bc = self.bettercap.clone();
        let sta_ttl = format!("{}", self.config.personality.sta_ttl);
        if let Err(e) = bc.run(&["set", "wifi.sta.ttl", &sta_ttl]).await {
            LOGGER.log_error("Agent", &format!("Failed to set wifi.sta.ttl: {e}"));
        }
        let bc = self.bettercap.clone();
        let min_rssi = format!("{}", self.config.personality.min_rssi);
        if let Err(e) = bc.run(&["set", "wifi.rssi.min", &min_rssi]).await {
            LOGGER.log_error("Agent", &format!("Failed to set wifi.rssi.min: {e}"));
        }
        let bc = self.bettercap.clone();
        let path = self.config.main.handshakes_path.clone();
        if let Err(e) = bc.run(&["set", "wifi.handshakes.file", &path]).await {
            LOGGER.log_error("Agent", &format!("Failed to set wifi.handshakes.file: {e}"));
        }
        let bc = self.bettercap.clone();
        let agg = false.to_string();
        if let Err(e) = bc.run(&["set", "wifi.handshakes.aggregate", agg.as_str()]).await {
            LOGGER.log_error("Agent", &format!("Failed to set wifi.handshakes.aggregate: {e}"));
        }
    }

    #[allow(clippy::future_not_send)]
    pub async fn start_monitor_mode(&mut self) {
        LOGGER.log_info("Agent", "Starting monitor mode...");
        let interface = &self.config.main.interface;
        let mon_start_cmd = &self.config.main.mon_start_cmd;
        let restart = !&self.config.main.no_restart;
        let mut is_starting = false;
        let mut has_iface = false;

        while !has_iface {
            let bc = self.bettercap.clone();
            let session_result = bc.session(None).await;

            if let Some(session) = session_result {
                for iface in session.interfaces {
                    if iface.name == *interface {
                        LOGGER.log_info("Agent", &format!("Found Monitor interface: {interface}"));
                        has_iface = true;
                        break;
                    }
                }

                if !has_iface {
                    if !is_starting && !mon_start_cmd.trim().is_empty() {
                        LOGGER.log_info("Agent", &format!("Starting monitor mode on {interface}"));
                        let cmd = mon_start_cmd.to_string();
                        let _ = tokio::task::spawn_blocking(move || {
                            match Command::new("sh").arg("-c").arg(&cmd).output() {
                                Ok(output) if output.status.success() => {
                                    LOGGER.log_info(
                                        "Agent",
                                        "Monitor mode command executed successfully"
                                    );
                                }
                                Ok(output) => {
                                    LOGGER.log_error(
                                        "Agent",
                                        &format!("Monitor mode failed: {output:?}")
                                    );
                                }
                                Err(e) => {
                                    LOGGER.log_error(
                                        "Agent",
                                        &format!("Failed to run monitor mode command: {e}")
                                    );
                                }
                            }
                        }).await;
                        is_starting = true;
                    } else {
                        LOGGER.log_info(
                            "Agent",
                            &format!("Monitor interface {interface} not found, waiting...")
                        );
                        tokio::time::sleep(Duration::from_secs(5)).await;
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
            if let Err(e) = self.bettercap.run(&["wifi.clear"]).await {
                LOGGER.log_error("Agent", &format!("Failed to clear wifi: {e}"));
            }
        } else if !wifi_running {
            LOGGER.log_info("Agent", "Starting WiFi module...");
            self.start_module(WIFI_RECON).await;
        }

        // Advertising logic here
    }

    #[allow(clippy::future_not_send)]
    pub async fn wait_for_bettercap(&self) {
        LOGGER.log_info("Agent", "Waiting for Bettercap to start...");
        loop {
            let ok = self.bettercap.session(None).await.is_some();
            if ok {
                return;
            }
            LOGGER.log_info("Agent", "Waiting for Bettercap...");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    // Full async startup sequence; call from a Tokio runtime.
    #[allow(clippy::future_not_send)]
    pub async fn start(&mut self) {
        self.wait_for_bettercap().await;
        self.setup_events().await;
        //self.set_starting();
        self.start_monitor_mode().await;
        //self.detect_channels();
        self.start_event_polling();
        self.automata.next_epoch();
        //self.start_session_fetcher();
        //self.set_ready();
    }

    fn detect_channels(&mut self) {
        self.config.personality.channels = self.supported_channels();
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

    pub fn start_event_polling(&mut self) {
        LOGGER.log_info("Agent", "Starting event polling thread...");
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel(100);
        self.event_inbox = Some(rx);

        let bettercap = self.bettercap.clone();
        let bc_for_ws = self.bettercap.clone();
        tokio::spawn(async move {
            bc_for_ws.run_websocket().await;
        });

        tokio::spawn(async move {
            let mut bc_rx = bettercap.subscribe_events();
            loop {
                match bc_rx.recv().await {
                    Ok(ev) => {
                        let _ = tx.send(ev).await;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        LOGGER.log_error("Agent", "Bettercap event channel closed");
                        break;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        LOGGER.log_warning("Agent", &format!("Lagged {n} events"));
                    }
                }
            }
        });
    }

    // Non-blocking drain of any queued events from the background thread.
    pub fn drain_events(&mut self) {
        if let Some(rx) = &mut self.event_inbox {
            // Collect events first to avoid holding an immutable borrow of self while mutably borrowing in on_event
            let mut events = Vec::new();
            loop {
                match rx.try_recv() {
                    Ok(ev) => events.push(ev),
                    Err(TryRecvError::Empty) => {
                        break;
                    }
                    Err(TryRecvError::Disconnected) => {
                        LOGGER.log_error("Agent", "Event inbox disconnected");
                        break;
                    }
                }
            }
            for ev in events {
                self.on_event(&ev);
            }
        }
    }

    fn on_event(&mut self, event: &str) {
        let mut found_handshake = false;
        let parsed: serde_json::Value = match serde_json::from_str(event) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Failed to parse event: {e}");
                return;
            }
        };
        let jmsg = match serde_json::to_value(&parsed) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Failed to serialize event: {e}");
                return;
            }
        };

        if
            let Some(tag) = jmsg.get("tag").and_then(|v| v.as_str()) &&
            tag == "wifi.client.handshake"
        {
            let _filename = jmsg
                .get("file")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let sta_mac = jmsg
                .get("station")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let ap_mac = jmsg
                .get("ap")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let key = format!("{ap_mac} -> {sta_mac}");

            if self.handshakes.contains_key(&key) {
                LOGGER.log_debug("AGENT", &format!("Handshake already exists for {key}"));
            } else {
                // Avoid network call here; consult cached AP/STA view
                let ap_and_sta = self.find_ap_sta_in_cached(sta_mac, ap_mac);

                match ap_and_sta {
                    None => {
                        LOGGER.log_warning(
                            "AGENT",
                            &format!("!!! Captured new Handshake: {ap_mac}")
                        );
                        self.last_pwned = Some(ap_mac.to_string());
                    }
                    Some((ap, _sta)) => {
                        self.last_pwned = if ap.hostname.is_empty() && ap.hostname != "<hidden>" {
                            Some(ap_mac.to_string())
                        } else {
                            Some(ap.hostname)
                        };

                        LOGGER.log_warning(
                            "AGENT",
                            &format!("Captured new handshake on channel {}", ap.channel)
                        );
                    }
                }

                found_handshake = true;
            }
            self.update_handshakes(u32::from(found_handshake));
        }
    }

    pub async fn get_access_points_by_channel(&mut self) -> Vec<(u8, Vec<AccessPoint>)> {
      let aps = self.get_access_points().await;
      let channels: &HashSet<u8> = &self.config.personality.channels.iter().cloned().collect();
      let mut grouped: HashMap<u8, Vec<AccessPoint>> = HashMap::new();

      LOGGER.log_debug("Agent", &format!("{} APS", aps.len()));

      for ap in aps {
          if channels.contains(&ap.channel) {
              grouped.entry(ap.channel).or_default().push(ap);
          }
      }

      LOGGER.log_debug(
          "Agent",
          &format!("Found {} populated channels", grouped.len()),
      );

      let mut grouped_vec: Vec<(u8, Vec<AccessPoint>)> = grouped.into_iter().collect();

      // Sort by population (descending), stable so channels with same count keep numeric order
      grouped_vec.sort_by(|a, b| {
          b.1.len()
              .cmp(&a.1.len())
              .then_with(|| a.0.cmp(&b.0))
      });

      grouped_vec
    }


    pub async fn recon(&mut self) {
        let mut recon_time = self.config.personality.recon_time;
        let max_inactive = self.config.personality.max_inactive_scale;
        let recon_multiplier = self.config.personality.recon_inactive_multiplier;
        let channels = &self.config.personality.channels;

        if self.automata.epoch.inactive_for >= max_inactive {
            recon_time *= recon_multiplier;
        }

        LOGGER.log_debug("RECON", "Starting Recon");

        if channels.is_empty() {
            self.current_channel = Some(0);
            LOGGER.log_debug("RECON", "Listening on all available channels.");
            if let Err(_e) = self.bettercap.run(&["wifi.recon.channel", "clear"]).await {
              LOGGER.log_debug("RECON", "Failed to set channels");
            }
        } else {
            use std::collections::HashSet;
            let channel_str = channels
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<HashSet<_>>() // deduplicate
                .into_iter()
                .collect::<Vec<_>>()
                .join(",");
            if let Err(e) = self.bettercap.run(&["wifi.recon.channel", &channel_str]).await {
                LOGGER.log_error("RECON", &format!("Failed to set recon channel: {e}"));
            }
        }

        LOGGER.log_debug("RECON", &format!("Recon time set to {recon_time} seconds"));

        self.automata.wait_for(recon_time, None).await;
    }

    #[allow(clippy::future_not_send)]
    pub async fn associate(&mut self, ap: &AccessPoint, mut throttle: Option<f32>) {
        if self.automata.is_stale() {
            LOGGER.log_debug(
                "AGENT",
                &format!("Automata is stale, skipping association to {}", ap.mac)
            );
            return;
        }

        if throttle.is_none() && self.config.personality.throttle_a.is_finite() {
            throttle = Some(self.config.personality.throttle_a);
        }

        if self.config.personality.associate && self.should_interact(&ap.mac) {
            LOGGER.log_debug(
                "AGENT",
                &format!(
                    "Sending association frame to {} ({}) on channel {} ({} clients), {} dBm",
                    ap.mac,
                    ap.hostname,
                    ap.channel,
                    ap.clients.len(),
                    ap.signal
                )
            );

            let mac = ap.mac.clone();
            match self.bettercap.run(&["wifi.associate", &mac]).await {
                Ok(()) => {
                    LOGGER.log_info(
                        "AGENT",
                        &format!(
                            "Associated with {} ({}) on channel {}",
                            ap.mac,
                            ap.hostname,
                            ap.channel
                        )
                    );
                    self.automata.epoch.track("association", Some(1));
                }
                Err(e) => {
                    self.automata.on_error(e.to_string().as_str());
                }
            }

            if let Some(throttle) = throttle {
                LOGGER.log_debug(
                    "AGENT",
                    &format!("Throttling association for {throttle} seconds")
                );
                sleep(Duration::from_secs_f32(throttle));
            }
        }
    }

    pub async fn deauth(&mut self, ap: &AccessPoint, sta: &Station, mut throttle: Option<f32>) {
        if self.automata.is_stale() {
            LOGGER.log_debug(
                "AGENT",
                &format!("Automata is stale, skipping deauth from {}", ap.mac)
            );
            return;
        }

        if throttle.is_none() && self.config.personality.throttle_d.is_finite() {
            throttle = Some(self.config.personality.throttle_d);
        }

        if self.config.personality.deauth && self.should_interact(&sta.mac) {
            LOGGER.log_debug(
                "AGENT",
                &format!(
                    "Sending deauth frame to {} ({}) on channel {} ({} clients), {} dBm",
                    ap.mac,
                    ap.hostname,
                    ap.channel,
                    ap.clients.len(),
                    ap.signal
                )
            );

            let mac = sta.mac.clone();
            let bc = self.bettercap.clone();
            match bc.run(&["wifi.deauth", &mac]).await {
                Ok(()) => {
                    LOGGER.log_info(
                        "AGENT",
                        &format!(
                            "Deauthenticated from {} ({}) on channel {}",
                            ap.mac,
                            ap.hostname,
                            ap.channel
                        )
                    );
                    self.automata.epoch.track("deauth", Some(1));
                }
                Err(e) => {
                    self.automata.on_error(e.to_string().as_str());
                }
            }

            if let Some(throttle) = throttle {
                LOGGER.log_debug("AGENT", &format!("Throttling deauth for {throttle} seconds"));
                sleep(Duration::from_secs_f32(throttle));
            }
        }
    }

    fn has_handshake(&self, bssid: &str) -> bool {
        self.handshakes.contains_key(bssid)
    }

    fn should_interact(&mut self, bssid: &str) -> bool {
        if self.has_handshake(bssid) {
            return false;
        } else if
            let std::collections::hash_map::Entry::Vacant(e) = self.history.entry(bssid.to_string())
        {
            e.insert(1);
            return true;
        }
        self.history.entry(bssid.to_string()).and_modify(|e| {
            *e += 1;
        });
        self.history[&bssid.to_string()] < self.config.personality.max_interactions
    }

    pub fn set_access_points(&mut self, aps: Vec<AccessPoint>) -> std::vec::Vec<AccessPoint> {
        self.access_points = aps;
        self.automata.epoch.observe(&self.access_points, &self.peers);
        self.access_points.clone()
    }

    pub async fn get_access_points(&mut self) -> Vec<AccessPoint> {
      let blacklist: Vec<String> = self
          .config
          .main
          .whitelist
          .iter()
          .map(|s| s.to_lowercase())
          .collect();

      let mut aps: Vec<AccessPoint> = Vec::new();

      if let Some(session) = self.bettercap.session(None).await {
          for ap in session.wifi.aps {
              LOGGER.log_debug("Agent", &format!("Got host {}", ap.hostname));

              if ap.encryption.is_empty() || ap.encryption.eq_ignore_ascii_case("OPEN") {
                  continue;
              }

              let mac = ap.mac.to_lowercase();
              let ssid = ap.hostname.to_lowercase();

              if blacklist.contains(&mac) || blacklist.contains(&ssid) {
                  continue;
              }

              aps.push(ap);
          }
      }

      aps.sort_by_key(|ap| -ap.signal);

      self.set_access_points(aps.clone());

      aps
    }

    #[must_use]
    pub const fn get_total_aps(&self) -> usize {
        self.access_points.len()
    }

    pub fn get_aps_on_channel(&self, channel: u8) -> Vec<AccessPoint> {
        self.access_points
            .iter()
            .filter(|ap| ap.channel == channel)
            .cloned()
            .collect()
    }

    fn update_handshakes(&mut self, new_shakes: u32) {
        if new_shakes > 0 {
            self.automata.epoch.track("handshake", Some(new_shakes));
        }

        let total = utils::total_unique_handshakes(&self.config.main.handshakes_path);
        let mut txt = format!("{} ({})", self.handshakes.len(), total);

        if let Some(ref last_pwned) = self.last_pwned {
            use std::fmt::Write;
            let _ = write!(txt, " [{last_pwned}]");
        }
    }

    #[must_use]
    pub fn supported_channels(&self) -> Vec<u8> {
        iface_channels(&self.config.main.interface)
    }

    #[must_use]
    pub async fn is_module_running(&self, module: &str) -> bool {
        match self.bettercap.session(None).await {
            Some(session) => session.modules.iter().any(|m| m.name == module && m.running),
            None => false,
        }
    }

    fn find_ap_sta_in(
        sta_mac: &str,
        ap_mac: &str,
        session: Option<BettercapSession>
    ) -> Option<(AccessPoint, Station)> {
        if let Some(session) = session {
            for ap in &session.wifi.aps {
                if ap.mac == ap_mac {
                    for sta in &ap.clients {
                        if sta.mac == sta_mac {
                            return Some((
                                ap.clone(),
                                Station {
                                    mac: sta.mac.clone(),
                                    hostname: sta.hostname.clone(),
                                    vendor: sta.vendor.clone(),
                                },
                            ));
                        }
                    }
                }
            }
        }
        None
    }

    // Find AP and station using the cached access_points list to avoid async calls in on_event
    fn find_ap_sta_in_cached(&self, sta_mac: &str, ap_mac: &str) -> Option<(AccessPoint, Station)> {
        for ap in &self.access_points {
            if ap.mac == ap_mac {
                for sta in &ap.clients {
                    if sta.mac == sta_mac {
                        return Some((
                            ap.clone(),
                            Station {
                                mac: sta.mac.clone(),
                                hostname: sta.hostname.clone(),
                                vendor: sta.vendor.clone(),
                            },
                        ));
                    }
                }
                // Some data sources might use stations instead of clients
                for sta in &ap.stations {
                    if sta.mac == sta_mac {
                        return Some((
                            ap.clone(),
                            Station {
                                mac: sta.mac.clone(),
                                hostname: sta.hostname.clone(),
                                vendor: sta.vendor.clone(),
                            },
                        ));
                    }
                }
            }
        }
        None
    }

    pub async fn set_channel(&mut self, channel: u8) {
        if self.automata.is_stale() {
            LOGGER.log_debug(
                "AGENT",
                &format!("Recon is stale, skipping channel switch to {channel}")
            );
            return;
        }

        LOGGER.log_debug("Agent", &format!("Attempting switch to Channel {channel}"));

        let mut wait = 0;
        if self.automata.epoch.did_deauth {
            wait = self.config.personality.hop_recon_time;
        } else if self.automata.epoch.did_associate {
            wait = self.config.personality.min_recon_time;
        }

        if channel != self.current_channel.unwrap_or(0) {
            if self.current_channel != Some(0) && wait > 0 {
                LOGGER.log_debug(
                    "AGENT",
                    &format!("Waiting {wait} seconds before switching channel")
                );
                self.automata.wait_for(wait, Some(false)).await;
            }
            let chs = channel.to_string();
            let bc = self.bettercap.clone();
            match bc.run(&["wifi.recon.channel", &chs]).await {
                Ok(()) => {
                    self.current_channel = Some(channel);
                    self.automata.epoch.track("hop", Some(1));
                    LOGGER.log_info("AGENT", &format!("Switched to channel {channel}"));
                }
                Err(e) => {
                    LOGGER.log_error("AGENT", &format!("Failed to switch channel: {e}"));
                }
            }
        }
    }

    #[allow(clippy::future_not_send)]
    pub async fn restart_module(&self, module: &str) {
        let bc = self.bettercap.clone();
        if let Err(e) = bc.run(&[module, "off"]).await {
            LOGGER.log_error("AGENT", &format!("Failed to stop module {module}: {e}"));
            return;
        }
        if let Err(e) = bc.run(&[module, "on"]).await {
            LOGGER.log_error("AGENT", &format!("Failed to start module {module}: {e}"));
        }
    }

    #[allow(clippy::future_not_send)]
    pub async fn stop_module(&self, module: &str) {
        let bc = self.bettercap.clone();
        if let Err(e) = bc.run(&[module, "off"]).await {
            LOGGER.log_error("AGENT", &format!("Failed to stop module {module}: {e}"));
        }
    }

    #[allow(clippy::future_not_send)]
    pub async fn start_module(&self, module: &str) {
        let bc = self.bettercap.clone();
        if let Err(e) = bc.run(&[module, "on"]).await {
            LOGGER.log_error("AGENT", &format!("Failed to start module {module}: {e}"));
        }
    }
}
