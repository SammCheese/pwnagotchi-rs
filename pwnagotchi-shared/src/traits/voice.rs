use crate::{
  mesh::peer::Peer,
  models::net::{AccessPoint, Station},
  sessions::lastsession::LastSession,
};

pub trait VoiceTrait: Send + Sync {
  fn custom(&self, msg: &str) -> String;
  fn default_line(&self) -> String;
  fn on_starting(&self) -> String;
  fn on_keys_generation(&self) -> String;
  fn on_normal(&self) -> String;
  fn on_free_channel(&self, channel: u8) -> String;
  fn on_reading_logs(&self, lines_so_far: u64) -> String;
  fn on_bored(&self) -> String;
  fn on_motivated(&self) -> String;
  fn on_demotivated(&self) -> String;
  fn on_sad(&self) -> String;
  fn on_angry(&self) -> String;
  fn on_excited(&self) -> String;
  fn on_new_peer(&self, peer: &Peer) -> String;
  fn on_lost_peer(&self, peer: &Peer) -> String;
  fn on_miss(&self, who: &str) -> String;
  fn on_grateful(&self) -> String;
  fn on_lonely(&self) -> String;
  fn on_napping(&self, secs: u64) -> String;
  fn on_shutdown(&self) -> String;
  fn on_awakening(&self) -> String;
  fn on_waiting(&self, secs: u64) -> String;
  fn on_assoc(&self, ap: &AccessPoint) -> String;
  fn on_deauth(&self, sta: &Station) -> String;
  fn on_handshakes(&self, num_shakes: u32) -> String;
  fn on_unread_messages(&self, count: u32) -> String;
  fn on_rebooting(&self) -> String;
  fn on_uploading(&self, to: &str) -> String;
  fn on_downloading(&self, from: &str) -> String;
  fn on_last_session_data(&self, last_session: &LastSession) -> String;
}
