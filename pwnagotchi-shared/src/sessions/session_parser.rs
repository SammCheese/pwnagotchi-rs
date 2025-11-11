use std::{
  fs,
  sync::{Arc, LazyLock},
  time::SystemTime,
};

use anyhow::Result;
use regex::Regex;
use time::{UtcDateTime, macros::format_description};

use crate::{
  mesh::peer::Peer,
  models::grid::Advertisement,
  sessions::session_stats::{EpochStats, PeerStats, SessionStats},
  traits::ui::ViewTrait,
};

const EPOCH_TOKEN: &str = "Epoch";
const TRAINING_TOKEN: &str = "training epoch";
const DEAUTH_TOKEN: &str = "deauthing ";
const ASSOC_TOKEN: &str = "sending association frame to ";
const HANDSHAKE_TOKEN: &str = "!!! captured new handshake ";
const PEER_TOKEN: &str = "detected unit ";

static EPOCH_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^.+\[Epoch (\d+)] (.+)$").unwrap());
static EPOCH_DATA_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([a-z_]+)=(\S+)").unwrap());
static PEER_RE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"detected unit (.+)@(.+) \(v.+\) on channel \d+ \(([\d\-]+) dBm\) \[sid:(.+) pwnd_tot:(\d+) uptime:(\d+)]").unwrap()
});

pub struct SessionParser;

pub fn parse_session_from_file(
  path: &str,
  view: Option<&Arc<dyn ViewTrait + Send + Sync>>,
) -> Result<SessionStats> {
  let content = fs::read_to_string(path)?;
  Ok(parse_session(&content, view))
}

fn parse_session(content: &str, view: Option<&Arc<dyn ViewTrait + Send + Sync>>) -> SessionStats {
  let mut stats = SessionStats::default();

  let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

  if let Some(view) = view {
    view.on_reading_logs(0);
  }

  for (line_index, line) in content.lines().enumerate() {
    let line = line.trim();
    if line.is_empty() || !line.starts_with('[') {
      continue;
    }

    if line_index % 100 == 0
      && let Some(view) = view
    {
      view.on_reading_logs(line_index.try_into().unwrap_or(0));
    }

    if let Some((ts, rest)) = line.split_once(']') {
      let ts_str = ts.trim_start_matches('[').trim();
      if let Ok(dt) = UtcDateTime::parse(ts_str, &format) {
        let system_time = SystemTime::from(dt);
        stats.stop = Some(system_time);
        if stats.start.is_none() {
          stats.start = Some(system_time);
        }
      }
      handle_line(rest.trim(), &mut stats);
    }
  }

  if let Some(view) = view {
    view.on_reading_logs(0);
  }

  stats
}

fn handle_line(line: &str, stats: &mut SessionStats) {
  if line.contains(DEAUTH_TOKEN) {
    stats.deauthed += 1;
  } else if line.contains(ASSOC_TOKEN) {
    stats.associated += 1;
  } else if line.contains(HANDSHAKE_TOKEN) {
    stats.handshakes += 1;
  } else if line.contains(TRAINING_TOKEN) {
    stats.epochs.train_epochs += 1;
  } else if line.contains(EPOCH_TOKEN) {
    stats.epochs.epochs += 1;
    handle_epoch_line(line, &mut stats.epochs);
  } else if line.contains(PEER_TOKEN) {
    handle_peer_line(line, &mut stats.peers);
  }
}

fn handle_epoch_line(line: &str, epochs: &mut EpochStats) {
  if let Some(caps) = EPOCH_RE.captures(line) {
    let epoch_data = caps.get(2).map_or("", |m| m.as_str());
    for cap in EPOCH_DATA_RE.captures_iter(epoch_data) {
      let key = cap.get(1).unwrap().as_str();
      let value = cap.get(2).unwrap().as_str();
      if key == "reward"
        && let Ok(reward) = value.parse::<f64>()
      {
        epochs.avg_reward += reward;
        if reward < epochs.min_reward {
          epochs.min_reward = reward;
        }
        if reward > epochs.max_reward {
          epochs.max_reward = reward;
        }
      }
    }
  }
}

fn handle_peer_line(line: &str, peers: &mut PeerStats) {
  if let Some(m) = PEER_RE.captures(line) {
    let name = m[1].to_string();
    let pubkey = m[2].to_string();
    let rssi: i16 = m[3].parse().unwrap_or(0);
    let sid = m[4].to_string();
    let pwnd_tot: u32 = m[5].parse().unwrap_or(0);

    peers.peers += 1;
    peers.last_peer = Some(Peer {
      session_id: sid,
      last_channel: 1,
      rssi,
      encounters: 1,
      first_met: None,
      first_seen: None,
      prev_seen: None,
      last_seen: None,
      adv: Advertisement {
        identity: pubkey.clone(),
        name,
        pwnd_total: pwnd_tot,
        ..Advertisement::default()
      },
    });

    *peers.history.entry(pubkey).or_insert(0) += 1;
  }
}
