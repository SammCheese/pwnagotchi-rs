pub trait SysData {
  fn get_temperature(&self) -> Result<f32, Box<dyn std::error::Error>>;
  fn get_uptime(&self) -> Result<u64, Box<dyn std::error::Error>>;
  fn get_cpu_usage(&self) -> Result<f32, Box<dyn std::error::Error>>;
  fn get_memory_usage(&self) -> Result<(f32, f32), Box<dyn std::error::Error>>; // (used, total)
  fn get_hostname(&self) -> Result<String, Box<dyn std::error::Error>>;
}