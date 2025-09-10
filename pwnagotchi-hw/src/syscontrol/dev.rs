use std::error::Error;

use pwnagotchi_shared::models::agent::RunningMode;

use crate::syscontrol::SysControl;

pub struct DevSysControl;

impl SysControl for DevSysControl {
  fn shutdown(&self) -> Result<(), Box<dyn Error>> {
    Ok(())
  }

  fn reboot(&self, _mode: Option<RunningMode>) -> Result<(), Box<dyn Error>> {
    Ok(())
  }

  fn restart(&self, _mode: RunningMode) -> Result<(), Box<dyn Error>> {
    Ok(())
  }
}
