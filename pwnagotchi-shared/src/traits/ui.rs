use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;
use tiny_skia::PixmapMut as RgbaImage;

use crate::{
  mesh::peer::Peer,
  models::net::{AccessPoint, Station},
  sessions::lastsession::LastSession,
  traits::general::{CoreModule, Dependencies},
};

pub trait Widget: Send + Sync {
  fn draw(&self, canvas: &mut RgbaImage);
  fn set_value(&mut self, value: &str);
  fn get_value(&self) -> &str;
}

#[async_trait::async_trait]
pub trait ServerTrait: Send + Sync + Dependencies {
  async fn start_server(&self) -> Result<(), String>;
  async fn stop_server(&self);
}

#[async_trait::async_trait]
pub trait UIRefresher: Send + Sync + Dependencies {
  async fn start(&self);
}

#[async_trait::async_trait]
pub trait ViewTrait: Send + Sync + CoreModule {
  fn set(&self, key: &str, value: String);
  fn get(&self, key: &str) -> Option<Arc<Mutex<Box<dyn Widget>>>>;
  fn on_state_change(&self, key: &str, callback: Box<dyn Fn(String, String) + Send + Sync>);
  fn on_starting(&self);
  fn on_manual_mode(&self, last_session: &LastSession);
  fn set_closest_peer(&self, peer: Option<&Peer>, total_peers: u32);
  fn on_new_peer(&self, peer: &Peer);
  fn on_keys_generation(&self);
  fn on_normal(&self);
  fn on_lost_peer(&self, peer: &Peer);
  fn on_free_channel(&self, channel: u8);
  fn on_reading_logs(&self, lines: u64);
  fn on_shutdown(&mut self);
  fn on_bored(&self);
  fn on_sad(&self);
  fn on_angry(&self);
  fn on_motivated(&self);
  fn on_demotivated(&self);
  fn on_excited(&self);
  fn on_assoc(&self, ap: &AccessPoint);
  fn on_deauth(&self, who: &Station);
  fn on_miss(&self, who: &AccessPoint);
  fn on_grateful(&self);
  fn on_lonely(&self);
  fn on_handshakes(&self, count: u32);
  fn on_unread_messages(&self, count: u32);
  fn on_uploading(&self, to: &str);
  fn on_rebooting(&self);
  fn on_custom(&self, text: &str);
  fn is_normal(&self) -> bool;
  fn update(&self, force: Option<bool>, new_data: Option<HashMap<String, String>>);
  async fn wait(&self, mut secs: f64, sleeping: bool);
  fn add_element(&self, key: &str, elem: Box<dyn Widget>);
  fn has_element(&self, key: &str) -> bool;
  fn on_render(&mut self, callback: Option<fn(&RgbaImage)>);
  async fn start_render_loop(&self);
}
