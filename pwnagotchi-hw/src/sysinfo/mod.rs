use std::error::Error;

mod dev;
mod pi;
mod portable;

pub use dev::DevSysInfo;
pub use pi::PiSysInfo;
pub use portable::PortableSysInfo;

pub trait SysInfo {
  fn get_temperature(&self, celsius: Option<bool>) -> Result<f32, Box<dyn Error>>;
  fn get_uptime(&self) -> Result<u64, Box<dyn Error>>;
  fn get_cpu_usage(&self) -> Result<f32, Box<dyn Error>>;
  fn get_memory_usage(&self) -> Result<f32, Box<dyn Error>>;
}
