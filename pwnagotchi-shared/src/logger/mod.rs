use std::{
  fs::{self, File, OpenOptions},
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
  file: Mutex<File>,
  debug_file: Mutex<File>,
}

impl Log {
  pub fn new(path: &str, debug_path: &str) -> Self {
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
    let debug_file = OpenOptions::new()
      .create(true)
      .append(true)
      .open(debug_path)
      .expect("Failed to open debug log file");

    Self {
      file: Mutex::new(file),
      debug_file: Mutex::new(debug_file),
    }
  }

  pub fn log(&self, origin: Option<&str>, message: &str, level: LogLevel) {
    let time = OffsetDateTime::now_utc();
    let entry = format!(
      "[{}] [{}] {} {}\n",
      time.format(&time::format_description::well_known::Rfc3339).unwrap(),
      format!("{:?}", level).to_uppercase(),
      origin.map_or("".to_string(), |o| format!("[{}]", o)),
      message
    );

    // Log Everything to debug log
    if let Ok(mut debug_file) = self.debug_file.lock()
      && let Err(e) = debug_file.write_all(entry.as_bytes())
    {
      eprintln!("Failed to write debug log entry: {e}");
    }

    // Only log Info and above to normal log
    if level >= LogLevel::Info
      && let Ok(mut file) = self.file.lock()
      && let Err(e) = file.write_all(entry.as_bytes())
    {
      eprintln!("Failed to write log entry: {e}");
    }
  }

  pub fn log_debug(&self, origin: &str, message: &str) {
    self.log(Some(origin), message, LogLevel::Debug);
  }

  pub fn log_info(&self, origin: &str, message: &str) {
    self.log(Some(origin), message, LogLevel::Info);
  }

  pub fn log_warning(&self, origin: &str, message: &str) {
    self.log(Some(origin), message, LogLevel::Warning);
  }

  pub fn log_error(&self, origin: &str, message: &str) {
    self.log(Some(origin), message, LogLevel::Error);
  }

  pub fn log_fatal(&self, origin: &str, message: &str) {
    self.log(Some(origin), message, LogLevel::Fatal);
  }
}

pub static LOGGER: std::sync::LazyLock<Log> = std::sync::LazyLock::new(|| {
  let log = Log::new(&config().log.path, &config().log.path_debug);
  log.log(None, "=========== STARTED ===========", LogLevel::Info);
  log
});
