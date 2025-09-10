use pwnagotchi_shared::models::agent::RunningMode;

mod dev;
mod pi;
mod portable;

pub use dev::DevSysControl;
pub use pi::PiSysControl;
pub use portable::PortableSysControl;

pub trait SysControl {
  fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>>;
  fn reboot(&self, mode: Option<RunningMode>) -> Result<(), Box<dyn std::error::Error>>;
  fn restart(&self, mode: RunningMode) -> Result<(), Box<dyn std::error::Error>>;
}
