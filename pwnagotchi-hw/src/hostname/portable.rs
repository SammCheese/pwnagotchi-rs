use std::fs;

use crate::hostname::HostnameManager;

pub struct PortableHostnameManager;

impl HostnameManager for PortableHostnameManager {
  fn get_hostname(&self) -> Result<String, String> {
    fs::read_to_string("").map_err(|e| e.to_string()).map(|s| s.trim().to_string())
  }

  fn set_hostname(&mut self, _new: &str) -> Result<(), String> {
    fs::write("", _new).map_err(|e| e.to_string())
  }
}
