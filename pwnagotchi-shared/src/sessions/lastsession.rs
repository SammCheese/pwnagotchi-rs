use std::sync::Arc;

use crate::{
  config::config,
  log::LOGGER,
  sessions::{session_parser::SessionParser, session_stats::SessionStats},
  traits::ui::ViewTrait,
};

pub struct LastSession {
  pub stats: Option<SessionStats>,
}

impl LastSession {
  pub fn new(view: &Arc<dyn ViewTrait + Send + Sync>) -> Self {
    let log_path = &config().log.path;
    let stats = SessionParser::from_file(log_path, view).ok();
    LOGGER
      .log_info("LastSession", &format!("Loaded Session data from log file: {:?}", stats.as_ref()));
    Self { stats }
  }

  pub fn reload(&mut self, view: &Arc<dyn ViewTrait + Send + Sync>) {
    let log_path = &config().log.path;
    self.stats = SessionParser::from_file(log_path, view).ok();
  }

  pub fn is_new(&self, last_saved_id: &str) -> bool {
    self.stats.as_ref().map(|s| s.id.as_str() != last_saved_id).unwrap_or(false)
  }
}
