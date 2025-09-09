use std::{
  fs::{self, OpenOptions},
  io::Write,
  path::Path,
  sync::Mutex,
};

use time::OffsetDateTime;

use crate::config::config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
  Debug = 0,
  Info = 1,
  Warning = 2,
  Error = 3,
  Fatal = 4,
}

pub struct Log {
  file: Mutex<std::fs::File>,
  min_level: LogLevel,
}

#[allow(dead_code)]
fn string_to_loglevel(level: &str) -> LogLevel {
  match level.to_lowercase().as_str() {
    "debug" => LogLevel::Debug,
    "warning" => LogLevel::Warning,
    "error" => LogLevel::Error,
    "fatal" => LogLevel::Fatal,
    _ => LogLevel::Info,
  }
}

impl Log {
  pub fn new(path: &str) -> Self {
    let p = Path::new(path);
    if let Some(parent) = p.parent()
      && !parent.exists()
      && let Err(e) = fs::create_dir_all(parent)
    {
      eprintln!("Failed to create log dir {parent:?}: {e}");
    }

    let file = OpenOptions::new()
      .create(true)
      .append(true)
      .open(path)
      .expect("Failed to open log file");

    Self {
      file: Mutex::new(file),
      min_level: if config().debug.enabled { LogLevel::Debug } else { LogLevel::Info },
    }
  }

  pub fn log(&self, origin: &str, message: &str, level: LogLevel) {
    if level < self.min_level {
      return;
    }

    let ts = OffsetDateTime::now_utc();
    let entry = format!(
      "[{}] [{}] [{}] {}\n",
      ts.format(&time::format_description::well_known::Rfc3339).unwrap(),
      format!("{:?}", level).to_uppercase(),
      origin,
      message
    );

    if let Ok(mut file) = self.file.lock()
      && let Err(e) = file.write_all(entry.as_bytes())
    {
      eprintln!("Failed to write log entry: {e}");
    }
  }

  pub fn log_debug(&self, origin: &str, message: &str) {
    self.log(origin, message, LogLevel::Debug);
  }

  pub fn log_info(&self, origin: &str, message: &str) {
    self.log(origin, message, LogLevel::Info);
  }

  pub fn log_warning(&self, origin: &str, message: &str) {
    self.log(origin, message, LogLevel::Warning);
  }

  pub fn log_error(&self, origin: &str, message: &str) {
    self.log(origin, message, LogLevel::Error);
  }

  pub fn log_fatal(&self, origin: &str, message: &str) {
    self.log(origin, message, LogLevel::Fatal);
  }
}

pub static LOGGER: std::sync::LazyLock<Log> = std::sync::LazyLock::new(|| {
  let log = Log::new(&config().log.path);
  log.log("", "=========== STARTED ===========", LogLevel::Info);
  log
});
