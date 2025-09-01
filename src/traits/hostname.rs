pub trait HostnameManager {
  fn get_hostname(&self) -> Result<String, Box<dyn std::error::Error>>;
  fn set_hostname(&self, hostname: &str) -> Result<(), Box<dyn std::error::Error>>;
}
