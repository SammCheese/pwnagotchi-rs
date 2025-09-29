use std::error::Error;

use crate::sysinfo::SysInfo;

pub struct DevSysInfo;

impl SysInfo for DevSysInfo {
  fn get_temperature(&self, celsius: Option<bool>) -> Result<f32, Box<dyn Error>> {
    let celsius = celsius.unwrap_or(true);
    let temp = if celsius { 42.0 } else { 42.0 * 9.0 / 5.0 + 32.0 };
    Ok(temp)
  }

  fn get_uptime(&self) -> Result<u64, Box<dyn Error>> {
    Ok(0)
  }

  fn get_cpu_usage(&self) -> Result<f32, Box<dyn Error>> {
    Ok(0.0)
  }

  fn get_memory_usage(&self) -> Result<f32, Box<dyn Error>> {
    Ok(0.0)
  }
}
