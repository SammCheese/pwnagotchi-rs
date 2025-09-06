#![allow(clippy::struct_field_names)]
#![allow(clippy::unused_self)]

use std::{borrow::Cow, collections::HashMap, fs};

use hex::ToHex;
use time::{UtcDateTime, macros::format_description};

use crate::core::{
  config::config,
  log::LOGGER,
  mesh::{advertiser::Advertisement, peer::Peer},
  ui::view::View,
  utils::format_duration_human,
  voice::Voice,
};

const EPOCH_TOKEN: &str = r"[epoch ";
const EPOCH_PARSER: &str = r"^.+\[epoch (\d+)] (.+)$";
const EPOCH_DATA_PARSER: &str = r"([a-z_]+)=(\S+)";
const TRAINING_TOKEN: &str = r" training epoch ";
const START_TOKEN: &str = r"Initializing Agent";
const DEAUTH_TOKEN: &str = r"deauthing ";
const ASSOC_TOKEN: &str = r"sending association frame to ";
const HANDSHAKE_TOKEN: &str = r"!!! captured new handshake ";
const PEER_TOKEN: &str = r"detected unit ";
const PEER_PARSER: &str = r"detected unit (.+)@(.+) \(v.+\) on channel \d+ \(([\d\-]+) dBm\) \[sid:(.+) pwnd_tot:(\d+) uptime:(\d+)]";
const LAST_SESSION_FILE: &str = "/root/pwnagotchi/.pwnagotchi-last-session";

#[derive(Debug, Clone)]

pub struct LastSession {
  pub voice: Voice,
  pub path: Cow<'static, str>,
  pub last_session: Vec<String>,
  pub last_session_id: String,
  pub last_saved_session_id: String,
  pub duration: String,
  pub duration_human: String,
  pub deauthed: u32,
  pub associated: u32,
  pub handshakes: u32,
  pub peers: u32,
  pub last_peer: Option<Peer>,
  pub epochs: u32,
  pub train_epochs: u32,
  pub min_reward: f64,
  pub max_reward: f64,
  pub avg_reward: f64,
  pub parsed: bool,
}

impl Default for LastSession {
  fn default() -> Self {
    Self {
      voice: Voice::new(),
      path: Cow::Borrowed(&config().main.log_path),
      last_session: Vec::new(),
      last_session_id: String::new(),
      last_saved_session_id: String::new(),
      duration: String::new(),
      duration_human: String::new(),
      deauthed: 0,
      associated: 0,
      handshakes: 0,
      peers: 0,
      last_peer: None,
      epochs: 0,
      train_epochs: 0,
      min_reward: 1000.0,
      max_reward: -1000.0,
      avg_reward: 0.0,
      parsed: false,
    }
  }
}

enum CacheType {
  I8(i8),
  Peer(Box<Peer>),
}

impl LastSession {
  pub fn new() -> Self {
    Self::default()
  }

  fn get_last_saved_session_id(&self) -> String {
    let saved = "";

    fs::read_to_string(LAST_SESSION_FILE)
      .unwrap_or_else(|_| saved.to_string())
      .trim()
      .to_string()
  }

  pub fn save_session_id(&mut self, session_id: &str) {
    fs::write(LAST_SESSION_FILE, session_id).unwrap_or(());

    self.last_saved_session_id = self.last_session_id.clone();
  }

  pub fn is_new(&self) -> bool {
    self.last_session_id != self.last_saved_session_id
  }

  pub fn parse(&mut self, ui: &View, skip: Option<bool>) {
    let skip = skip.unwrap_or(false);

    if skip {
      LOGGER.log_debug("Session", "Skipping parsing of the last session logs...");
      self.parsed = true;
      return;
    }

    LOGGER.log_debug("Session", "Reading last session logs...");

    let mut lines: Vec<String> = vec![];

    match fs::read_to_string(&*self.path) {
      Ok(content) => {
        let content_lines: Vec<String> =
          content.lines().map(|line| line.trim().to_string()).rev().collect();

        for line in content_lines {
          if !line.is_empty() && !line.starts_with('[') {
            continue;
          }

          lines.push(line.clone());

          if line.contains(START_TOKEN) {
            break;
          }

          let lines_so_far = lines.len() as u64;

          if lines_so_far.is_multiple_of(100) {
            ui.on_reading_logs(lines_so_far);
          }
        }

        lines.reverse();

        if lines.is_empty() {
          lines.push("Initial Session".to_string());
        }

        self.last_session.clone_from(&lines);
        self.last_session_id = md5::compute(lines[0].as_bytes()).encode_hex::<String>();
        self.last_saved_session_id = self.get_last_saved_session_id();
        self.parse_stats();
        self.parsed = true;
      }
      Err(e) => {
        LOGGER.log_error("Session", &format!("Failed to read session log file: {e}"));

        self.parsed = true;
      }
    }
    LOGGER.log_info(
      "Session",
      &format!(
        "Parsed last session: {} (saved: {}, new: {})",
        self.last_session_id,
        self.last_saved_session_id,
        self.is_new()
      ),
    );
  }

  fn parse_stats(&mut self) {
    let mut started_at: Option<UtcDateTime> = None;
    let mut stopped_at: Option<UtcDateTime> = None;
    let mut cache = HashMap::<String, CacheType>::new();

    let epoch_data_re = match regex::Regex::new(EPOCH_DATA_PARSER) {
      Ok(re) => re,
      Err(e) => {
        LOGGER.log_error("SESSION", &format!("Failed to compile EPOCH_DATA_PARSER regex: {e}"));
        return;
      }
    };

    let epoch_re = match regex::Regex::new(EPOCH_PARSER) {
      Ok(re) => re,
      Err(e) => {
        LOGGER.log_error("SESSION", &format!("Failed to compile EPOCH_PARSER regex: {e}"));
        return;
      }
    };

    let peer_re = match regex::Regex::new(PEER_PARSER) {
      Ok(re) => re,
      Err(e) => {
        LOGGER.log_error("SESSION", &format!("Failed to compile PEER_PARSER regex: {e}"));
        return;
      }
    };

    for line in self.last_session.clone() {
      let parts = line.split(']').collect::<Vec<&str>>();

      if parts.len() < 2 {
        continue;
      }

      let line_timestamp = parts[0].trim_start_matches('[').trim();
      let line = parts[1..].join("]");

      let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
      stopped_at = UtcDateTime::parse(line_timestamp, &format).ok();

      if started_at.is_none() {
        started_at = stopped_at;
      }

      self.handle_line(&line, &mut cache, &epoch_re, &epoch_data_re, &peer_re);
    }

    self.finalize_stats(started_at, stopped_at);
  }

  fn handle_line(
    &mut self,
    line: &str,
    cache: &mut HashMap<String, CacheType>,
    epoch_re: &regex::Regex,
    epoch_data_re: &regex::Regex,
    peer_re: &regex::Regex,
  ) {
    match line {
      line if line.contains(DEAUTH_TOKEN) && !cache.contains_key(line) => {
        self.deauthed += 1;
        cache.insert(line.to_string(), CacheType::I8(1));
      }
      line if line.contains(ASSOC_TOKEN) && !cache.contains_key(line) => {
        self.associated += 1;
        cache.insert(line.to_string(), CacheType::I8(1));
      }
      line if line.contains(HANDSHAKE_TOKEN) && !cache.contains_key(line) => {
        self.handshakes += 1;
        cache.insert(line.to_string(), CacheType::I8(1));
      }
      line if line.contains(TRAINING_TOKEN) => {
        self.train_epochs += 1;
      }
      line if line.contains(EPOCH_TOKEN) => {
        self.epochs += 1;
        self.handle_epoch_line(line, epoch_re, epoch_data_re);
      }
      line if line.contains(PEER_TOKEN) => {
        self.handle_peer_line(line, cache, peer_re);
      }
      _ => {}
    }
  }

  fn handle_epoch_line(
    &mut self,
    line: &str,
    epoch_re: &regex::Regex,
    epoch_data_re: &regex::Regex,
  ) {
    if let Some(caps) = epoch_re.captures(line) {
      let epoch_data = caps.get(2).map_or("", |m| m.as_str());

      for cap in epoch_data_re.captures_iter(epoch_data) {
        let key = cap.get(1);
        let value = cap.get(2);

        if let (Some(key), Some(value)) = (key, value)
          && key.as_str() == "reward"
          && let Ok(reward) = value.as_str().parse::<f64>()
        {
          self.avg_reward += reward;

          if reward < self.min_reward {
            self.min_reward = reward;
          } else if reward > self.max_reward {
            self.max_reward = reward;
          }
        }
      }
    }
  }

  fn handle_peer_line(
    &mut self,
    line: &str,
    cache: &mut HashMap<String, CacheType>,
    peer_re: &regex::Regex,
  ) {
    if let Some(m) = peer_re.captures(line)
      && m.len() != 0
    {
      let name = m.get(1).map_or("", |m| m.as_str()).to_string();
      let pubkey = m.get(2).map_or("", |m| m.as_str()).to_string();
      let rssi = m.get(3).map_or("", |m| m.as_str()).to_string();
      let sid = m.get(4).map_or("", |m| m.as_str()).to_string();
      let pwnd_tot = m.get(5).map_or("0", |m| m.as_str()).parse::<u32>().unwrap_or(0);

      if !cache.contains_key(&pubkey) {
        self.last_peer = Some(Peer {
          session_id: sid,
          last_channel: 1,
          rssi: rssi.parse::<i16>().unwrap_or(0),
          encounters: 1,
          first_met: None,
          first_seen: None,
          prev_seen: None,
          last_seen: None,
          adv: Advertisement {
            identity: pubkey,
            name,
            pwnd_total: pwnd_tot,
            ..Advertisement::default()
          },
        });
      } else if let Some(last_peer) = &self.last_peer
        && let Some(entry) = cache.get_mut(&pubkey)
      {
        *entry = CacheType::Peer(Box::new(last_peer.clone()));
      }
    }
  }

  fn finalize_stats(&mut self, started_at: Option<UtcDateTime>, stopped_at: Option<UtcDateTime>) {
    if let (Some(start), Some(stop)) = (started_at, stopped_at) {
      let duration = stop.unix_timestamp() - start.unix_timestamp();

      self.duration = format!("{duration}");
    } else {
      self.duration = "0".into();
    }

    let seconds = self.duration.parse::<u64>().unwrap_or(0);

    self.duration_human = format_duration_human(std::time::Duration::from_secs(seconds));

    self.avg_reward /= if self.epochs > 0 { f64::from(self.epochs) } else { 1.0 };
  }
}
