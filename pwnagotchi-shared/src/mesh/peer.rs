use std::fmt::Debug;

use time::OffsetDateTime;

use crate::{
  config::config,
  log::LOGGER,
  models::grid::{Advertisement, PeerResponse},
};

fn parse_rfc3339(dt: &str) -> OffsetDateTime {
  if dt == "0001-01-01T00:00:00Z" {
    return OffsetDateTime::now_utc();
  }

  // Strip fractional seconds if present
  let trimmed = dt.split('.').next().unwrap_or(dt);

  // Parse with custom format (no fractional seconds, no offset parsing here)
  let format =
    time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]Z").unwrap();

  OffsetDateTime::parse(trimmed, &format).unwrap_or_else(|_| OffsetDateTime::now_utc()) // fallback if parsing fails
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Peer {
  pub first_met: Option<OffsetDateTime>,
  pub first_seen: Option<OffsetDateTime>,
  pub prev_seen: Option<OffsetDateTime>,
  pub last_seen: Option<OffsetDateTime>,
  pub encounters: u32,
  pub session_id: String,
  pub last_channel: u8,
  pub rssi: i16,
  pub adv: Advertisement,
}

impl Peer {
  /// Creates a new Peer from a `PeerResponse`.
  ///
  /// # Errors
  ///
  /// This function will use a fallback if the time format description cannot be
  /// parsed.
  pub fn new(data: &PeerResponse) -> Self {
    let now = OffsetDateTime::now_utc();
    let format = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]")
      .unwrap_or_else(|_| Vec::new());
    let just_met = now.format(&format).unwrap_or_else(|_| now.to_string());

    let data = data.clone();

    Self {
      first_met: Some(parse_rfc3339(&data.met_at.unwrap_or_else(|| just_met.clone()))),
      first_seen: Some(parse_rfc3339(&data.detected_at.unwrap_or_else(|| just_met.clone()))),
      prev_seen: Some(parse_rfc3339(&data.prev_seen_at.unwrap_or(just_met))),
      last_seen: Some(now),
      encounters: data.encounters.unwrap_or(0),
      session_id: data.session_id.unwrap_or_default(),
      last_channel: data.channel.unwrap_or(1),
      rssi: data.rssi.unwrap_or(0),
      adv: data.advertisement.unwrap_or_default(),
    }
  }

  pub fn update(&mut self, new: &Self) {
    if self.name() != new.name() {
      LOGGER.log_info(
        "PEER",
        &format!("Peer changed name from {} to {}", self.full_name(), new.full_name()),
      );
    }

    if self.session_id != new.session_id {
      LOGGER.log_info(
        "PEER",
        &format!(
          "Peer {} changed session ID from {} to {}",
          self.full_name(),
          self.session_id,
          new.session_id
        ),
      );
    }

    self.adv = new.adv.clone();
    self.rssi = new.rssi;
    self.session_id.clone_from(&new.session_id);
    self.last_seen = Some(time::OffsetDateTime::now_utc());
    self.prev_seen = new.prev_seen;
    self.first_met = new.first_met;
    self.encounters = new.encounters;
  }

  pub fn inactive_for(&self) -> time::Duration {
    self
      .last_seen
      .map_or(time::Duration::ZERO, |last_seen| time::OffsetDateTime::now_utc() - last_seen)
  }

  pub const fn is_first_encounter(&self) -> bool {
    self.encounters == 1
  }

  pub fn is_good_friend(&self) -> bool {
    self.encounters >= config().personality.bond_encounters_factor
  }

  pub fn face(&self) -> String {
    self.adv.face.clone()
  }

  pub fn name(&self) -> String {
    self.adv.name.clone()
  }

  pub fn identity(&self) -> String {
    self.adv.identity.clone()
  }

  pub fn full_name(&self) -> String {
    format!("{}@{}", self.adv.name, self.adv.identity)
  }

  pub fn version(&self) -> String {
    self.adv.version.clone()
  }

  pub const fn pwnd_run(&self) -> u32 {
    self.adv.pwnd_run
  }

  pub const fn pwnd_total(&self) -> u32 {
    self.adv.pwnd_total
  }

  pub const fn uptime(&self) -> u32 {
    self.adv.uptime
  }

  pub const fn epoch(&self) -> u32 {
    self.adv.epoch
  }

  pub const fn is_closer(&self, other: &Self) -> bool {
    self.rssi > other.rssi
  }
}
