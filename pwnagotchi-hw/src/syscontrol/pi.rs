use std::{error::Error, thread::sleep};

use pwnagotchi_shared::{logger::LOGGER, models::agent::RunningMode};

use crate::syscontrol::SysControl;

pub struct PiSysControl;

impl SysControl for PiSysControl {
  fn shutdown(&self) -> Result<(), Box<dyn Error>> {
    Ok(())
  }

  fn reboot(&self, mode: Option<RunningMode>) -> Result<(), Box<dyn Error>> {
    if mode.is_some() {
      LOGGER.log_warning("Pwnagotchi", &format!("Rebooting in {mode:?} mode..."));
    } else {
      LOGGER.log_warning("Pwnagotchi", "Rebooting...");
    }

    if RunningMode::Auto == mode.unwrap_or(RunningMode::Auto) {
      let _ = std::process::Command::new("touch").arg("/root/.pwnagotchi-auto").status();
    } else if RunningMode::Manual == mode.unwrap_or(RunningMode::Auto) {
      let _ = std::process::Command::new("touch").arg("/root/.pwnagotchi-manual").status();
    }

    LOGGER.log_warning("Pwnagotchi", "Syncing....");

    let _ = std::process::Command::new("sync").status();
    let _ = std::process::Command::new("shutdown").arg("-r").arg("now").status();
    Ok(())
  }

  fn restart(&self, mode: RunningMode) -> Result<(), Box<dyn Error>> {
    LOGGER.log_warning("Pwnagotchi", &format!("Restarting in {mode:?} mode..."));

    if RunningMode::Auto == mode {
      let _ = std::process::Command::new("touch").arg("/root/.pwnagotchi-auto").status();
    } else {
      let _ = std::process::Command::new("touch").arg("/root/.pwnagotchi-manual").status();
    }

    let _ = std::process::Command::new("service").arg("bettercap").arg("restart").status();
    sleep(std::time::Duration::from_secs(1));
    let _ = std::process::Command::new("service").arg("pwnagotchi").arg("restart").status();
    Ok(())
  }
}
