use crate::core::config::Config;


pub struct Server {
  pub config: Config,
  pub address: String,
  pub port: u16,
}

impl Default for Server {
  fn default() -> Self {
    let config = Config::default();
    let address = "127.0.0.1".into();
    let port = 8080;
    Self {
      config,
      address,
      port,
    }
  }
}

impl Server {
  pub const fn new(config: Config, address: String, port: u16) -> Self {
    Self {
      config,
      address,
      port,
    }
  }

  pub const fn start(&self) {
    
  }

  pub const fn stop(&self) {
    // Stop the server
  }
}