use crate::core::config::{config};


pub struct Server {
  pub address: String,
  pub port: u16,
}

impl Default for Server {
  fn default() -> Self {
    let address = config().ui.web.address.clone();
    let port = config().ui.web.port;
    Self {
      address,
      port,
    }
  }
}

impl Server {
  pub fn new() -> Self {
    Self {
      address: config().ui.web.address.clone(),
      port: config().ui.web.port,
    }
  }

  pub const fn start(&self) {
    
  }

  pub const fn stop(&self) {
    // Stop the server
  }
}