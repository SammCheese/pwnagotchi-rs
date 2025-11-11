use std::sync::Arc;

use crate::{
  config::config_read,
  sessions::{session_parser::parse_session_from_file, session_stats::SessionStats},
  traits::ui::ViewTrait,
};

pub struct LastSession {
  pub stats: Option<SessionStats>,
  pub view: Option<Arc<dyn ViewTrait + Send + Sync>>,
}

impl Default for LastSession {
  fn default() -> Self {
    Self::new()
  }
}

impl LastSession {
  pub fn new() -> Self {
    let log_path = &config_read().log.path;
    let stats = parse_session_from_file(log_path, None).ok();
    Self { stats, view: None }
  }

  pub fn reparse(&mut self) {
    let log_path = &config_read().log.path;
    self.stats = parse_session_from_file(log_path, self.view.as_ref()).ok();
  }

  pub fn reload(&mut self, view: Option<&Arc<dyn ViewTrait + Send + Sync>>) {
    let log_path = &config_read().log.path;
    self.stats = parse_session_from_file(log_path, view).ok();
  }

  pub fn is_new(&self, last_saved_id: &str) -> bool {
    self.stats.as_ref().map(|s| s.id.as_str() != last_saved_id).unwrap_or(false)
  }
}
