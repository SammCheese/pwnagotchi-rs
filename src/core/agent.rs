use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::{collections::HashMap, fs, time::Duration};
use tokio::time::sleep as a_sleep;

use crate::core::bettercap::{BettercapCommand, BettercapHandle};
use crate::core::config::config;
use crate::core::models::net::{AccessPoint, Handshake, Peer, Station};
use crate::core::session::LastSession;
use crate::core::ui::state::StateValue;
use crate::core::{
    automata::Automata,
    log::LOGGER,
    models::bettercap::BettercapSession,
    utils::{self},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunningMode {
    Auto,
    Manual,
    Ai,
    Custom,
}

const WIFI_RECON: &str = "wifi.recon";
const RECOVERY_FILE: &str = "/root/.pwnagotchi-recovery";

pub struct Agent {
    pub automata: Automata,
    pub bettercap: Arc<BettercapHandle>,
    pub started_at: std::time::SystemTime,
    pub lastsession: LastSession,
    pub current_channel: u8,
    pub total_aps: u32,
    pub aps_on_channel: u32,
    pub supported_channels: Vec<u8>,
    pub peers: Vec<Peer>,

    pub access_points: Vec<AccessPoint>,
    pub last_pwned: Option<String>,
    pub history: HashMap<String, u32>,
    pub handshakes: HashMap<String, Handshake>,
    pub mode: RunningMode,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
    pub event_type: String,
    pub data: HashMap<String, String>,
}

impl Agent {
    pub fn new(bc: Arc<BettercapHandle>) -> Self {
        Self::initialize();
        Self {
            automata: Automata::new(),
            bettercap: bc,
            lastsession: LastSession::new(),
            started_at: std::time::SystemTime::now(),
            current_channel: 0,
            total_aps: 0,
            aps_on_channel: 0,
            supported_channels: vec![1, 6, 11],
            access_points: Vec::new(),
            peers: Vec::new(),
            last_pwned: None,
            history: HashMap::new(),
            handshakes: HashMap::new(),
            mode: RunningMode::Manual,
        }
    }

    fn initialize() {
        LOGGER.log_info("Agent", "Initializing Agent");

        let handshakes_path = &config().main.handshakes_path;
        if fs::metadata(handshakes_path).is_err()
            && let Err(e) = fs::create_dir_all(handshakes_path)
        {
            LOGGER.log_fatal("Agent", &format!("Failed to create handshakes dir: {e}"));
        }
    }

    pub async fn setup_events(&mut self) {
        LOGGER.log_debug("Agent", "Setting up Bettercap events...");
        for event in &config().bettercap.silence {
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.bettercap
                .send_command(BettercapCommand::Run {
                    args: vec![
                        "set".to_string(),
                        "events.ignore".to_string(),
                        event.clone(),
                    ],
                    respond_to: tx,
                })
                .await;

            if let Err(e) = rx.await {
                LOGGER.log_error(
                    "Agent",
                    &format!("Failed to set events.ignore for {event}: {e}"),
                );
            }
        }
    }

    pub async fn reset_wifi_settings(&self) {
        let interface = config().main.interface.clone();
        let ap_ttl = format!("{}", config().personality.ap_ttl);
        let sta_ttl = format!("{}", config().personality.sta_ttl);
        let min_rssi = format!("{}", config().personality.min_rssi);

        let (ap_tx, ap_rx) = tokio::sync::oneshot::channel();
        let (sta_tx, sta_rx) = tokio::sync::oneshot::channel();

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.bettercap
            .send_command(BettercapCommand::Run {
                args: vec![
                    "set".to_string(),
                    "wifi.interface".to_string(),
                    interface.clone(),
                ],
                respond_to: tx,
            })
            .await;
        if let Err(e) = rx.await {
            LOGGER.log_error("Agent", &format!("Failed to set wifi.interface: {e}"));
        }

        self.bettercap
            .send_command(BettercapCommand::Run {
                args: vec!["set".to_string(), "wifi.ap.ttl".to_string(), ap_ttl.clone()],
                respond_to: ap_tx,
            })
            .await;
        if let Err(e) = ap_rx.await {
            LOGGER.log_error("Agent", &format!("Failed to set wifi.ap.ttl: {e}"));
        }
        self.bettercap
            .send_command(BettercapCommand::Run {
                args: vec![
                    "set".to_string(),
                    "wifi.sta.ttl".to_string(),
                    sta_ttl.clone(),
                ],
                respond_to: sta_tx,
            })
            .await;
        if let Err(e) = sta_rx.await {
            LOGGER.log_error("Agent", &format!("Failed to set wifi.sta.ttl: {e}"));
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.bettercap
            .send_command(BettercapCommand::Run {
                args: vec![
                    "set".to_string(),
                    "wifi.rssi.min".to_string(),
                    min_rssi.clone(),
                ],
                respond_to: tx,
            })
            .await;
        if let Err(e) = rx.await {
            LOGGER.log_error("Agent", &format!("Failed to set wifi.rssi.min: {e}"));
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        let path = config().main.handshakes_path.clone();
        self.bettercap
            .send_command(BettercapCommand::Run {
                args: vec![
                    "set".to_string(),
                    "wifi.handshakes.file".to_string(),
                    path.clone(),
                ],
                respond_to: tx,
            })
            .await;
        if let Err(e) = rx.await {
            LOGGER.log_error("Agent", &format!("Failed to set wifi.handshakes.file: {e}"));
        }
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.bettercap
            .send_command(BettercapCommand::Run {
                args: vec![
                    "set".to_string(),
                    "wifi.handshakes.aggregate".to_string(),
                    "false".to_string(),
                ],
                respond_to: tx,
            })
            .await;
        if let Err(e) = rx.await {
            LOGGER.log_error(
                "Agent",
                &format!("Failed to set wifi.handshakes.aggregate: {e}"),
            );
        }
    }

    pub async fn start_monitor_mode(&mut self) {
        let interface = &config().main.interface;
        let mon_start_cmd = &config().main.mon_start_cmd;
        let no_restart = config().main.no_restart;
        let mut is_starting = false;
        let mut has_iface = false;

        while !has_iface {
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.bettercap
                .send_command(BettercapCommand::GetSession { respond_to: tx })
                .await;

            if let Ok(Some(session)) = rx.await {
                for iface in session.interfaces {
                    if iface.name == *interface {
                        LOGGER.log_info("Agent", &format!("Found Monitor interface: {interface}"));
                        has_iface = true;
                        break;
                    }
                }

                if !is_starting && !mon_start_cmd.trim().is_empty() {
                    let cmd = mon_start_cmd.clone();
                    let status = tokio::process::Command::new("sh")
                        .arg("-c")
                        .arg(&cmd)
                        .status()
                        .await;
                    match status {
                        Ok(status) if status.success() => {
                            LOGGER.log_info("Agent", "Monitor mode command executed successfully");
                        }
                        Ok(status) => {
                            LOGGER.log_error(
                                "Agent",
                                &format!("Monitor mode command failed with status: {status}"),
                            );
                        }
                        Err(e) => {
                            LOGGER.log_error(
                                "Agent",
                                &format!("Failed to run monitor mode command: {e}"),
                            );
                        }
                    }
                }
                if !has_iface && !is_starting {
                    is_starting = true;
                    LOGGER.log_info(
                        "Agent",
                        &format!("Waiting for interface {interface} to appear..."),
                    );
                }
            } else {
                LOGGER.log_warning(
                    "Agent",
                    "Bettercap session not available, cannot check interfaces",
                );
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }

        self.reset_wifi_settings().await;

        let wifi_running = self.is_module_running("wifi").await;

        // Ensure the device is ready
        tokio::time::sleep(Duration::from_secs(2)).await;

        if wifi_running && !no_restart {
            LOGGER.log_debug("Agent", "Restarting WiFi module...");
            self.restart_module(WIFI_RECON).await;
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.bettercap
                .send_command(BettercapCommand::Run {
                    args: vec!["wifi.clear".to_string()],
                    respond_to: tx,
                })
                .await;
            if let Err(e) = rx.await {
                LOGGER.log_error("Agent", &format!("Failed to clear wifi: {e}"));
            }
        } else if !wifi_running {
            LOGGER.log_debug("Agent", "Starting WiFi module...");
            self.start_module(WIFI_RECON).await;
        }

        //self.advertiser.start_advertising()
    }

    pub async fn wait_for_bettercap(&self) {
        loop {
            let (tx, rx) = tokio::sync::oneshot::channel();

            self.bettercap
                .send_command(BettercapCommand::GetSession { respond_to: tx })
                .await;
            if let Ok(Some(_session)) = rx.await {
                tokio::time::sleep(Duration::from_secs(1)).await;
                return;
            }
            LOGGER.log_info("Agent", "Waiting for Bettercap...");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn start(&mut self) {
        self.wait_for_bettercap().await;
        self.setup_events().await;
        self.automata.set_starting();
        self.start_monitor_mode().await;

        self.automata.next_epoch();
        self.automata.set_ready();
    }

    pub async fn stop(&mut self) {
        LOGGER.log_info("Agent", "Stopping agent...");
        self.stop_module(WIFI_RECON).await;
        self.reset();
        LOGGER.log_info("Agent", "Agent stopped.");
    }

    pub fn set_mode(&mut self, mode: RunningMode) {
        self.mode = mode;
        match self.mode {
            RunningMode::Auto => {
                self.automata
                    .view
                    .set("mode", StateValue::Text("auto".into()));
            }
            RunningMode::Manual => {
                self.automata
                    .view
                    .set("mode", StateValue::Text("manual".into()));
            }
            RunningMode::Ai => {
                self.automata
                    .view
                    .set("mode", StateValue::Text("ai".into()));
            }
            RunningMode::Custom => {
                self.automata
                    .view
                    .set("mode", StateValue::Text("custom".into()));
            }
        }
    }

    pub fn reset(&mut self) {
        self.started_at = std::time::SystemTime::now();
        self.current_channel = 0;
        self.total_aps = 0;
        self.aps_on_channel = 0;
        self.access_points.clear();
        self.last_pwned = None;
        self.history.clear();
        self.handshakes.clear();
    }

    pub async fn get_access_points_by_channel(&mut self) -> Vec<(u8, Vec<AccessPoint>)> {
        let aps = self.get_access_points().await;
        let channels: &HashSet<u8> = &config().personality.channels.iter().copied().collect();
        let mut grouped: HashMap<u8, Vec<AccessPoint>> = HashMap::new();

        LOGGER.log_debug("Agent", &format!("{} APS", aps.len()));

        for ap in aps {
            if channels.contains(&ap.channel) || channels.is_empty() {
                grouped.entry(ap.channel).or_default().push(ap);
            }
        }

        LOGGER.log_debug(
            "Agent",
            &format!("Found {} populated channels", grouped.len()),
        );

        let mut grouped_vec: Vec<(u8, Vec<AccessPoint>)> = grouped.into_iter().collect();

        // Sort by population (descending), stable so channels with same count keep numeric order
        grouped_vec.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));

        grouped_vec
    }

    pub async fn recon(&mut self) {
        let mut recon_time = config().personality.recon_time;
        let max_inactive = config().personality.max_inactive_scale;
        let recon_multiplier = config().personality.recon_inactive_multiplier;
        let channels = &config().personality.channels;

        if self.automata.epoch.inactive_for >= max_inactive {
            recon_time *= recon_multiplier;
        }

        LOGGER.log_debug("RECON", "Starting Recon");
        self.automata
            .view
            .set("channel", StateValue::Text("*".into()));

        if channels.is_empty() {
            self.current_channel = 0;
            LOGGER.log_info("RECON", "Listening on all available channels.");
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.bettercap
                .send_command(BettercapCommand::Run {
                    args: vec!["wifi.recon.channel".to_string(), "clear".to_string()],
                    respond_to: tx,
                })
                .await;
            if let Err(_e) = rx.await {
                LOGGER.log_error("RECON", "Failed to set channels");
            }
        } else {
            let channel_str = channels
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<HashSet<_>>() // deduplicate
                .into_iter()
                .collect::<Vec<_>>()
                .join(",");
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.bettercap
                .send_command(BettercapCommand::Run {
                    args: vec!["wifi.recon.channel".to_string(), channel_str],
                    respond_to: tx,
                })
                .await;
            if let Err(e) = rx.await {
                LOGGER.log_error("RECON", &format!("Failed to set recon channel: {e}"));
            }
        }

        LOGGER.log_debug("RECON", &format!("Recon time set to {recon_time} seconds"));

        self.automata.wait_for(recon_time, Some(false)).await;
    }

    #[allow(clippy::future_not_send)]
    pub async fn associate(&mut self, ap: &AccessPoint, mut throttle: Option<f32>) {
        if self.automata.is_stale() {
            LOGGER.log_debug(
                "AGENT",
                &format!("Recon is stale, skipping association to {}", ap.mac),
            );
            return;
        }

        if throttle.is_none() && config().personality.throttle_a.is_finite() {
            throttle = Some(config().personality.throttle_a);
        }

        if config().personality.associate && self.should_interact(&ap.mac) {
            self.automata.view.on_assoc(ap);

            LOGGER.log_info(
                "AGENT",
                &format!(
                    "sending association frame to {} ({}) on channel {} ({} clients), {} dBm",
                    ap.mac,
                    ap.hostname,
                    ap.channel,
                    ap.clients.len(),
                    ap.rssi
                ),
            );

            let mac = ap.mac.clone();
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.bettercap
                .send_command(BettercapCommand::Run {
                    args: vec!["wifi.assoc".to_string(), mac],
                    respond_to: tx,
                })
                .await;

            match rx.await {
                Ok(res) => match res {
                    Ok(()) => {
                        LOGGER.log_info(
                            "AGENT",
                            &format!(
                                "Associated with {} ({}) on channel {}",
                                ap.mac, ap.hostname, ap.channel
                            ),
                        );
                        self.automata.epoch.track("association", Some(1));
                    }
                    Err(e) => {
                        self.automata.on_error(ap, e.to_string().as_str());
                    }
                },
                Err(e) => {
                    self.automata.on_error(ap, e.to_string().as_str());
                }
            }

            if let Some(throttle) = throttle {
                LOGGER.log_debug(
                    "AGENT",
                    &format!("Throttling association for {throttle} seconds"),
                );
                a_sleep(Duration::from_secs_f32(throttle)).await;
            }
            self.automata.view.on_normal();
        }
    }

    pub async fn deauth(&mut self, ap: &AccessPoint, sta: &Station, mut throttle: Option<f32>) {
        if self.automata.is_stale() {
            LOGGER.log_debug(
                "AGENT",
                &format!("Recon is stale, skipping deauth {}", sta.mac),
            );
            return;
        }

        if throttle.is_none() && config().personality.throttle_d.is_finite() {
            throttle = Some(config().personality.throttle_d);
        }

        if config().personality.deauth && self.should_interact(&sta.mac) {
            self.automata.view.on_deauth(sta);

            LOGGER.log_info(
                "AGENT",
                &format!(
                    "deauthing {} ({}) on channel {} ({} clients), {} dBm",
                    ap.mac,
                    ap.hostname,
                    ap.channel,
                    ap.clients.len(),
                    ap.rssi
                ),
            );

            let mac = sta.mac.clone();
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.bettercap
                .send_command(BettercapCommand::Run {
                    args: vec!["wifi.deauth".to_string(), mac],
                    respond_to: tx,
                })
                .await;

            match rx.await {
                Ok(res) => match res {
                    Ok(()) => {
                        LOGGER.log_info(
                            "AGENT",
                            &format!(
                                "Deauthenticated {} from {} on channel {}",
                                sta.mac, ap.hostname, ap.channel
                            ),
                        );
                        self.automata.epoch.track("deauth", Some(1));
                    }
                    Err(e) => {
                        self.automata.on_error(ap, e.to_string().as_str());
                    }
                },
                Err(e) => {
                    self.automata.on_error(ap, e.to_string().as_str());
                }
            }

            if let Some(throttle) = throttle {
                LOGGER.log_debug(
                    "AGENT",
                    &format!("Throttling deauth for {throttle} seconds"),
                );
                tokio::time::sleep(Duration::from_secs_f32(throttle)).await;
            }

            self.automata.view.on_normal();
        }
    }

    /*const fn restart(&mut self) {
      // TODO
    }

    const fn reboot(&mut self) {
      // TODO
    }*/

    fn has_handshake(&self, bssid: &str) -> bool {
        self.handshakes.contains_key(bssid)
    }

    fn should_interact(&mut self, bssid: &str) -> bool {
        if self.has_handshake(bssid) {
            return false;
        } else if let std::collections::hash_map::Entry::Vacant(e) =
            self.history.entry(bssid.to_string())
        {
            e.insert(1);
            return true;
        }
        self.history.entry(bssid.to_string()).and_modify(|e| {
            *e += 1;
        });
        self.history[&bssid.to_string()] < config().personality.max_interactions
    }

    pub fn set_access_points(&mut self, aps: Vec<AccessPoint>) -> std::vec::Vec<AccessPoint> {
        self.access_points = aps;
        self.automata
            .epoch
            .observe(&self.access_points, &self.peers);
        self.access_points.clone()
    }

    pub async fn get_access_points(&mut self) -> Vec<AccessPoint> {
        let blacklist: Vec<String> = config()
            .main
            .whitelist
            .iter()
            .map(|s| s.to_lowercase())
            .collect();

        let mut aps: Vec<AccessPoint> = Vec::new();

        let (tx, rx) = tokio::sync::oneshot::channel();

        self.bettercap
            .send_command(BettercapCommand::GetSession { respond_to: tx })
            .await;

        if let Ok(Some(session)) = rx.await {
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

        aps.sort_by_key(|ap| -ap.rssi);

        self.set_access_points(aps.clone());

        aps
    }

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

    pub fn update_handshakes(&mut self, new_shakes: u32) {
        if new_shakes > 0 {
            self.automata.epoch.track("handshake", Some(new_shakes));
        }

        let total = utils::total_unique_handshakes(&config().main.handshakes_path);
        let mut txt = format!("{} ({})", self.handshakes.len(), total);

        if let Some(ref last_pwned) = self.last_pwned {
            use std::fmt::Write;
            let _ = write!(txt, " [{last_pwned}]");
        }
    }

    #[allow(clippy::future_not_send)]
    pub async fn is_module_running(&self, module: &str) -> bool {
        let (tx, rx) = tokio::sync::oneshot::channel();

        self.bettercap
            .send_command(BettercapCommand::GetSession { respond_to: tx })
            .await;

        match rx.await {
            Ok(Some(session)) => session
                .modules
                .iter()
                .any(|m| m.name == module && m.running),
            _ => false,
        }
    }

    pub fn find_ap_sta_in(
        sta_mac: &str,
        ap_mac: &str,
        session: Option<BettercapSession>,
    ) -> Option<(AccessPoint, Station)> {
        if let Some(session) = session {
            for ap in &session.wifi.aps {
                if ap.mac == ap_mac {
                    for sta in &ap.clients {
                        if sta.mac == sta_mac {
                            return Some((ap.clone(), sta.clone()));
                        }
                    }
                }
            }
        }
        None
    }

    fn find_ap_sta_in_cached(&self, sta_mac: &str, ap_mac: &str) -> Option<(AccessPoint, Station)> {
        for ap in &self.access_points {
            if ap.mac == ap_mac {
                for sta in &ap.clients {
                    if sta.mac == sta_mac {
                        return Some((ap.clone(), sta.clone()));
                    }
                }
                for sta in &ap.clients {
                    if sta.mac == sta_mac {
                        return Some((ap.clone(), sta.clone()));
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
                &format!("Recon is stale, skipping channel switch to {channel}"),
            );
            return;
        }

        LOGGER.log_debug("Agent", &format!("Attempting switch to Channel {channel}"));

        let mut wait = 0;
        if self.automata.epoch.did_deauth {
            wait = config().personality.hop_recon_time;
        } else if self.automata.epoch.did_associate {
            wait = config().personality.min_recon_time;
        }

        if channel != self.current_channel {
            if self.current_channel != 0 && wait > 0 {
                LOGGER.log_debug(
                    "AGENT",
                    &format!("Waiting {wait} seconds before switching channel"),
                );
                self.automata.wait_for(wait, None).await;
            }
            let chs = channel.to_string();
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.bettercap
                .send_command(BettercapCommand::Run {
                    args: vec!["wifi.recon.channel".to_string(), chs],
                    respond_to: tx,
                })
                .await;

            match rx.await {
                Ok(Ok(())) => {
                    self.current_channel = channel;
                    self.automata.epoch.track("hop", Some(1));
                    self.automata
                        .view
                        .set("channel", StateValue::Number(channel.into()));
                    LOGGER.log_info("AGENT", &format!("Switched to channel {channel}"));
                }
                Ok(Err(e)) => {
                    LOGGER.log_error("AGENT", &format!("Failed to switch channel: {e}"));
                }
                Err(e) => {
                    LOGGER.log_error("AGENT", &format!("Failed to receive response: {e}"));
                }
            }
        }
    }

    #[allow(clippy::future_not_send)]
    pub async fn restart_module(&self, module: &str) {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.bettercap
            .send_command(BettercapCommand::Run {
                args: vec![module.to_string(), "off".to_string()],
                respond_to: tx,
            })
            .await;

        if let Err(e) = rx.await {
            LOGGER.log_error("AGENT", &format!("Failed to stop module {module}: {e}"));
            return;
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.bettercap
            .send_command(BettercapCommand::Run {
                args: vec![module.to_string(), "on".to_string()],
                respond_to: tx,
            })
            .await;

        if let Err(e) = rx.await {
            LOGGER.log_error("AGENT", &format!("Failed to start module {module}: {e}"));
        }
    }

    #[allow(clippy::future_not_send)]
    pub async fn stop_module(&self, module: &str) {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.bettercap
            .send_command(BettercapCommand::Run {
                args: vec![module.to_string(), "off".to_string()],
                respond_to: tx,
            })
            .await;

        if let Err(e) = rx.await {
            LOGGER.log_error("AGENT", &format!("Failed to stop module {module}: {e}"));
        }
    }

    #[allow(clippy::future_not_send)]
    pub async fn start_module(&self, module: &str) {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.bettercap
            .send_command(BettercapCommand::Run {
                args: vec![module.to_string(), "on".to_string()],
                respond_to: tx,
            })
            .await;

        if let Err(e) = rx.await {
            LOGGER.log_error("AGENT", &format!("Failed to start module {module}: {e}"));
        }
    }
}
