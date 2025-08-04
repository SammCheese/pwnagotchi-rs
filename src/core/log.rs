use once_cell::sync::Lazy;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
}
pub struct Log {
  file: String
}

impl Log {
  pub fn new(file: &str) -> Self {
    Log {
      file: file.to_string()
    }
  }

  fn log(&self, origin: &str, message: &str, level: LogLevel) {
    let level = match level {
      LogLevel::Debug => "DEBUG",
      LogLevel::Info => "INFO",
      LogLevel::Warning => "WARNING",
      LogLevel::Error => "ERROR",
      LogLevel::Fatal => "FATAL",
    };
    let log_entry = format!("[{}]: [{}] {}\n", origin, level, message);
    std::fs::write(&self.file, log_entry)
      .unwrap_or_else(|e| eprintln!("Failed to write to log file: {}", e));
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
    std::process::exit(1);
  }
}

impl Default for Log {
  fn default() -> Self {
    Log {
      file: "pwnagotchi.log".into()
    }
  }
}


pub static LOGGER: Lazy<Log> = Lazy::new(|| {
    Log::new("/var/log/pwnagotchi.log")
});
