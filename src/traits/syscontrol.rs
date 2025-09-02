pub trait SysControl {
  fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>>;

  fn reboot(&self) -> Result<(), Box<dyn std::error::Error>>;

  fn restart_service(&self, service_name: &str) -> Result<(), Box<dyn std::error::Error>>;
}
