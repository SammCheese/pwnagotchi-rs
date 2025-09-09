use std::{fs, time::Duration};

use crate::models::{
  agent::RunningMode,
  net::{AccessPoint, Station},
};

pub fn total_unique_handshakes(handshakes_path: &str) -> u32 {
  let mut total = 0;

  if let Ok(entries) = fs::read_dir(handshakes_path) {
    for entry in entries.filter_map(Result::ok) {
      if entry.path().extension().is_some_and(|ext| ext == "pcap") {
        total += 1;
      }
    }
  }

  total
}

pub fn random_choice<T>(choices: &[T]) -> String
where
  T: AsRef<str>,
{
  fastrand::choice(choices).map(|s| s.as_ref().to_string()).unwrap_or_default()
}

pub fn format_duration_human(duration: Duration) -> String {
  let seconds = duration.as_secs();
  let minutes = seconds / 60;
  let hours = minutes / 60;
  format!("{:02}:{:02}:{:02}", hours, minutes % 60, seconds % 60)
}

pub fn hostname_or_mac(ap: &AccessPoint) -> &str {
  if ap.hostname.trim().is_empty() || ap.hostname.contains("<hidden>") {
    &ap.mac
  } else {
    &ap.hostname
  }
}

pub fn sta_hostname_or_mac(sta: &Station) -> &str {
  if sta.hostname.trim().is_empty() || sta.hostname.contains("<hidden>") {
    &sta.mac
  } else {
    &sta.hostname
  }
}

pub fn mode_to_str(mode: RunningMode) -> String {
  match mode {
    RunningMode::Ai => "AI".into(),
    RunningMode::Manual => "MANU".into(),
    RunningMode::Auto => "AUTO".into(),
    RunningMode::Custom => "CUST".into(),
  }
}
