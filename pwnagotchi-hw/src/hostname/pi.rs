use std::{fs, process::Command};

use pwnagotchi_shared::logger::LOGGER;

use crate::hostname::HostnameManager;

pub struct PiHostnameManager;

impl HostnameManager for PiHostnameManager {
  fn get_hostname(&self) -> Result<String, String> {
    fs::read_to_string("/etc/hostname")
      .map_err(|e| e.to_string())
      .map(|s| s.trim().to_string())
  }

  fn set_hostname(&mut self, new: &str) -> Result<(), String> {
    if new.trim().is_empty() {
      return Err("Hostname cannot be empty".to_string());
    }

    if new.len() > 25 || new.trim().len() < 3 {
      return Err(
        "Hostname must be less than 25 characters and at least 3 characters long".to_string(),
      );
    }

    if !new.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
      return Err("Hostname must contain only alphanumeric characters and hyphens".to_string());
    }

    let current_hostname = self.get_hostname()?;
    if current_hostname == new {
      return Ok(());
    }

    let prev = fs::read("/etc/hosts").map_err(|e| e.to_string())?;

    let new_hosts = String::from_utf8_lossy(&prev)
      .lines()
      .map(|line| {
        if line.contains(&current_hostname) {
          line.replace(&current_hostname, new)
        } else {
          line.to_string()
        }
      })
      .collect::<Vec<String>>()
      .join("\n");
    fs::write("/etc/hosts", new_hosts).map_err(|e| e.to_string())?;

    LOGGER.log_info("Pwnagotchi", &format!("Setting hostname to '{}'", new));
    let _ = fs::write("/etc/hostname", new).map_err(|e| e.to_string());

    let status = Command::new("hostname").arg(new).status().map_err(|e| e.to_string())?;
    let _: () = if !status.success() {
      return Err("Failed to set hostname via `hostname` command".to_string());
    };
    Ok(())
  }
}
