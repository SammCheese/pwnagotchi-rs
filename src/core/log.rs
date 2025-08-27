use std::{ fs::OpenOptions, io::Write };

use crate::core::config::config;

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
    Self {
      file: file.to_string(),
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
    let config_level = string_to_loglevel(config().main.loglevel.as_str());

    if level < &config_level {
      return;
    }

    let log_entry = format!("[{}] [{}]: [{}] {}\n", chrono::Utc::now(), origin, level_str, message);

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
    Self {
      file: config().main.log_path.clone(),
    }
  }
}

pub static LOGGER: std::sync::LazyLock<Log> = std::sync::LazyLock::new(|| {
  Log::new(config().main.log_path.clone().as_str())
});
