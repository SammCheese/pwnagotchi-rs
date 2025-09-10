use crate::hostname::HostnameManager;

pub struct DevHostnameManager {
  pub hostname: String,
}

impl HostnameManager for DevHostnameManager {
  fn get_hostname(&self) -> Result<String, String> {
    Ok(self.hostname.clone())
  }

  fn set_hostname(&mut self, new: &str) -> Result<(), String> {
    self.hostname = new.to_string();
    Ok(())
  }
}
