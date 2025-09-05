#![allow(clippy::must_use_candidate)]

use std::{fs, process::Command, time::Duration};

use regex::Regex;

use crate::core::{
  agent::RunningMode,
  config::config,
  models::net::{AccessPoint, Station},
  ui::view::FaceType,
};

pub fn iface_channels(name: &str) -> Vec<u8> {
  let phy_out = match Command::new("/sbin/iw").args(["dev", name, "info"]).output() {
    Ok(output) => output,
    Err(e) => {
      eprintln!("Failed to execute iw dev info: {e}");

      return vec![];
    }
  };

  if !phy_out.status.success() {
    return vec![];
  }

  let phy_str = String::from_utf8_lossy(&phy_out.stdout);

  let phy_id = phy_str
    .lines()
    .find_map(|line| {
      if line.trim_start().starts_with("wiphy") { line.split_whitespace().nth(1) } else { None }
    })
    .unwrap_or("");

  if phy_id.is_empty() {
    return vec![];
  }

  let chan_out = match Command::new("/sbin/iw").args([&format!("phy{phy_id}"), "channels"]).output()
  {
    Ok(output) => output,
    Err(e) => {
      eprintln!("Failed to execute iw phy channels: {e}");

      return vec![];
    }
  };

  if !chan_out.status.success() {
    return vec![];
  }

  let chan_str = String::from_utf8_lossy(&chan_out.stdout);

  let re = match Regex::new(r"\[(\d+)\]") {
    Ok(regex) => regex,
    Err(e) => {
      eprintln!("Failed to compile regex: {e}");

      return vec![];
    }
  };

  let mut channels = Vec::new();

  for cap in re.captures_iter(&chan_str) {
    if let Some(m) = cap.get(1)
      && let Ok(ch) = m.as_str().parse::<u8>()
    {
      channels.push(ch);
    }
  }

  channels
}

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

pub enum STAP<'a> {
  Station(&'a Station),
  AccessPoint(&'a AccessPoint),
}

pub fn hostname_or_mac<'a>(station: &'a STAP<'a>) -> &'a str {
  match station {
    STAP::Station(sta) => {
      if sta.hostname.trim().is_empty() || sta.hostname.contains("<hidden>") {
        &sta.mac
      } else {
        &sta.hostname
      }
    }
    STAP::AccessPoint(ap) => {
      if ap.hostname.trim().is_empty() || ap.hostname.contains("<hidden>") {
        &ap.mac
      } else {
        &ap.hostname
      }
    }
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

pub fn face_to_string(face: &FaceType) -> String {
  let faces = &config().faces;

  let face_str = match face {
    FaceType::LookR => &faces.look_r,
    FaceType::LookL => &faces.look_l,
    FaceType::LookRHappy => &faces.look_r_happy,
    FaceType::LookLHappy => &faces.look_l_happy,
    FaceType::Sleep => &faces.sleep,
    FaceType::Sleep2 => &faces.sleep2,
    FaceType::Awake => &faces.awake,
    FaceType::Bored => &faces.bored,
    FaceType::Intense => &faces.intense,
    FaceType::Cool => &faces.cool,
    FaceType::Happy => &faces.happy,
    FaceType::Grateful => &faces.grateful,
    FaceType::Excited => &faces.excited,
    FaceType::Motivated => &faces.motivated,
    FaceType::Demotivated => &faces.demotivated,
    FaceType::Smart => &faces.smart,
    FaceType::Lonely => &faces.lonely,
    FaceType::Sad => &faces.sad,
    FaceType::Angry => &faces.angry,
    FaceType::Friend => &faces.friend,
    FaceType::Broken => &faces.broken,
    FaceType::Debug => &faces.debug,
    FaceType::Upload => &faces.upload,
    FaceType::Upload1 => &faces.upload1,
    FaceType::Upload2 => &faces.upload2,
  };

  face_str.to_string()
}
