use std::{sync::Arc, time::Duration};

use pwnagotchi_shared::{config::config, log::LOGGER};

use crate::{
  agent::{is_module_running, start_module},
  bettercap::BettercapCommand,
  traits::bettercapcontroller::BettercapController,
};

const WIFI_RECON: &str = "wifi.recon";

pub async fn perform_bettercap_setup(bc: &Arc<dyn BettercapController>) {
  wait_for_bettercap(bc).await;
  setup_events(bc).await;
  start_monitor_mode(bc).await;
}

async fn setup_events(bc: &Arc<dyn BettercapController>) {
  LOGGER.log_debug("Agent", "Setting up Bettercap events...");

  for event in &config().bettercap.silence {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = bc.send(BettercapCommand::run(format!("events.ignore {event}"), Some(tx))).await;

    if let Err(e) = rx.await {
      LOGGER.log_error("Agent", &format!("Failed to set events.ignore for {event}: {e}"));
    }
  }
}

async fn reset_wifi_settings(bc: &Arc<dyn BettercapController>) {
  let interface = &config().main.iface;
  let ap_ttl = format!("{}", config().personality.ap_ttl);
  let sta_ttl = format!("{}", config().personality.sta_ttl);
  let min_rssi = format!("{}", config().personality.min_rssi);
  let (ap_tx, ap_rx) = tokio::sync::oneshot::channel();
  let (sta_tx, sta_rx) = tokio::sync::oneshot::channel();
  let (tx, rx) = tokio::sync::oneshot::channel();

  let _ = bc
    .send(BettercapCommand::run(format!("set wifi.interface {interface}"), Some(tx)))
    .await;

  if let Err(e) = rx.await {
    LOGGER.log_error("Agent", &format!("Failed to set wifi.interface: {e}"));
  }

  let _ = bc
    .send(BettercapCommand::run(format!("set wifi.ap.ttl {ap_ttl}"), Some(ap_tx)))
    .await;
  if let Err(e) = ap_rx.await {
    LOGGER.log_error("Agent", &format!("Failed to set wifi.ap.ttl: {e}"));
  }

  let _ = bc
    .send(BettercapCommand::run(format!("set wifi.sta.ttl {sta_ttl}"), Some(sta_tx)))
    .await;
  if let Err(e) = sta_rx.await {
    LOGGER.log_error("Agent", &format!("Failed to set wifi.sta.ttl: {e}"));
  }

  let (tx, rx) = tokio::sync::oneshot::channel();
  let _ = bc
    .send(BettercapCommand::run(format!("set wifi.rssi.min {min_rssi}"), Some(tx)))
    .await;
  if let Err(e) = rx.await {
    LOGGER.log_error("Agent", &format!("Failed to set wifi.rssi.min: {e}"));
  }

  let (tx, rx) = tokio::sync::oneshot::channel();
  let path = &config().bettercap.handshakes;
  let _ = bc
    .send(BettercapCommand::run(format!("set wifi.handshakes.file {path}"), Some(tx)))
    .await;
  if let Err(e) = rx.await {
    LOGGER.log_error("Agent", &format!("Failed to set wifi.handshakes.file: {e}"));
  }

  let (tx, rx) = tokio::sync::oneshot::channel();
  let _ = bc
    .send(BettercapCommand::run("set wifi.handshakes.aggregate false", Some(tx)))
    .await;
  if let Err(e) = rx.await {
    LOGGER.log_error("Agent", &format!("Failed to set wifi.handshakes.aggregate: {e}"));
  }
}

async fn start_monitor_mode(bc: &Arc<dyn BettercapController>) {
  let interface = &config().main.iface;
  let mon_start_cmd = &config().main.mon_start_cmd;
  let no_restart = &config().main.no_restart;
  let mut is_starting = false;
  let mut has_iface = false;

  while !has_iface {
    if let Ok(Some(session)) = bc.session().await {
      for iface in session.interfaces {
        if iface.name == *interface {
          LOGGER.log_info("Agent", &format!("Found Monitor interface: {interface}"));
          has_iface = true;
          break;
        }
      }

      if !is_starting && !mon_start_cmd.trim().is_empty() {
        let cmd = mon_start_cmd.as_ref();
        let status = tokio::process::Command::new("sh").arg("-c").arg(cmd).status().await;

        match status {
          Ok(status) if status.success() => {
            LOGGER.log_info("Agent", "Monitor mode command executed successfully");
          }
          Ok(status) => {
            LOGGER
              .log_error("Agent", &format!("Monitor mode command failed with status: {status}"));
          }
          Err(e) => {
            LOGGER.log_error("Agent", &format!("Failed to run monitor mode command: {e}"));
          }
        }
      }

      if !has_iface && !is_starting {
        is_starting = true;

        LOGGER.log_info("Agent", &format!("Waiting for interface {interface} to appear..."));
      }
    } else {
      LOGGER.log_warning("Agent", "Bettercap session not available, cannot check interfaces");
    }
    tokio::time::sleep(Duration::from_secs(5)).await;
  }

  reset_wifi_settings(bc).await;

  let wifi_running = is_module_running(bc, "wifi").await;

  // Ensure the device is ready
  tokio::time::sleep(Duration::from_secs(1)).await;

  if wifi_running && !no_restart {
    LOGGER.log_debug("Agent", "Restarting WiFi module...");
    start_module(bc, WIFI_RECON).await;
    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = bc.send(BettercapCommand::run("wifi.clear", Some(tx))).await;
    if let Err(e) = rx.await {
      LOGGER.log_error("Agent", &format!("Failed to clear wifi: {e}"));
    }
  } else if !wifi_running {
    LOGGER.log_debug("Agent", "Starting WiFi module...");
    start_module(bc, WIFI_RECON).await;
  }

  //self.advertiser.start_advertising()
}

async fn wait_for_bettercap(bc: &Arc<dyn BettercapController>) {
  loop {
    let (tx, rx) = tokio::sync::oneshot::channel();

    let _ = bc.send(BettercapCommand::GetSession { respond_to: tx }).await;

    if let Ok(Some(_session)) = rx.await {
      tokio::time::sleep(Duration::from_secs(1)).await;
      return;
    }
    LOGGER.log_info("Agent", "Waiting for Bettercap...");
    tokio::time::sleep(Duration::from_secs(1)).await;
  }
}
