use std::sync::Arc;

use crate::{hostname::*, syscontrol::*, sysinfo::*};

pub enum Mode {
  Pi,
  Portable,
  Dev,
}

pub struct Backend {
  pub hostname: Arc<dyn HostnameManager>,
  pub sysinfo: Arc<dyn SysInfo>,
  pub syscontrol: Arc<dyn SysControl>,
}

impl Backend {
  pub fn new(mode: Mode) -> Self {
    match mode {
      Mode::Pi => Self {
        hostname: Arc::new(PiHostnameManager),
        sysinfo: Arc::new(PiSysInfo),
        syscontrol: Arc::new(PiSysControl),
      },
      Mode::Portable => Self {
        hostname: Arc::new(PortableHostnameManager),
        sysinfo: Arc::new(PortableSysInfo),
        syscontrol: Arc::new(PortableSysControl),
      },
      Mode::Dev => Self {
        hostname: Arc::new(DevHostnameManager { hostname: "pwnagotchi".to_string() }),
        sysinfo: Arc::new(DevSysInfo),
        syscontrol: Arc::new(DevSysControl),
      },
    }
  }
}
