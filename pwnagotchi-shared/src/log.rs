use std::{
  fs::{self, OpenOptions},
  io::Write,
};

use time::UtcDateTime;

use crate::config::config;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LogLevel {
  Debug,
  Info,
  Warning,
  Error,
  Fatal,
}

pub struct Log {
  file: String,
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
  pub fn new(file: &str) -> Self {
    let logfile = Self { file: file.to_string() };

    logfile.initialize();

    logfile
  }

  fn initialize(&self) {
    let path = std::path::Path::new(&self.file);

    if let Some(parent) = path.parent()
      && !parent.exists()
      && let Err(e) = fs::create_dir_all(parent)
    {
      eprintln!("Failed to create log directory: {e}");
    }
  }

  fn log(&self, origin: &str, message: &str, level: &LogLevel) {
    let level_str = match level {
      LogLevel::Debug => "DEBUG",
      LogLevel::Info => "INFO",
      LogLevel::Warning => "WARNING",
      LogLevel::Error => "ERROR",
      LogLevel::Fatal => "FATAL",
    };

    let config_level = match &config().debug.enabled {
      true => LogLevel::Debug,
      false => LogLevel::Info,
    };

    if level < &config_level {
      return;
    }

    let log_entry = format!("[{}] [{}]: [{}] {}\n", UtcDateTime::now(), origin, level_str, message);

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.file) {
      if let Err(e) = file.write_all(log_entry.as_bytes()) {
        eprintln!("Failed to write to log file: {e}");
      }
    } else {
      eprintln!("Failed to open log file: {}", &self.file);
    }
  }

  pub fn log_debug(&self, origin: &str, message: &str) {
    self.log(origin, message, &LogLevel::Debug);
  }

  pub fn log_info(&self, origin: &str, message: &str) {
    self.log(origin, message, &LogLevel::Info);
  }

  pub fn log_warning(&self, origin: &str, message: &str) {
    self.log(origin, message, &LogLevel::Warning);
  }

  pub fn log_error(&self, origin: &str, message: &str) {
    self.log(origin, message, &LogLevel::Error);
  }

  pub fn log_fatal(&self, origin: &str, message: &str) {
    self.log(origin, message, &LogLevel::Fatal);

    std::process::exit(1);
  }
}

impl Default for Log {
  fn default() -> Self {
    Self { file: config().log.path.to_string() }
  }
}

pub static LOGGER: std::sync::LazyLock<Log> = std::sync::LazyLock::new(|| {
  let log = Log::new(&config().log.path);
  log.log_info("", "#=========== STARTED ===========#");
  log
});
