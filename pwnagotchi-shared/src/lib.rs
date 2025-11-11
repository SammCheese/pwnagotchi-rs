pub mod ai;
pub mod config;
pub mod identity;
pub mod logger;
pub mod voice;

pub mod traits {
  pub mod agent;
  pub mod automata;
  pub mod bettercap;
  pub mod epoch;
  pub mod events;
  pub mod general;
  pub mod grid;
  pub mod logger;
  pub mod plugins;
  pub mod ui;
}

pub mod types {
  pub mod epoch;
  pub mod events;
  pub mod grid;
  pub mod hooks;
  pub mod ui;
}

pub mod sessions {
  pub mod lastsession;
  pub mod manager;
  pub mod recovery;
  pub mod session;
  pub mod session_parser;
  pub mod session_stats;
}

pub mod mesh {
  pub mod peer;
}

pub mod models {
  pub mod agent;
  pub mod bettercap;
  pub mod epoch;
  pub mod grid;
  pub mod net;
}

pub mod utils {
  pub mod agent;
  pub mod faces;
  pub mod general;
  pub mod hooks;
  pub mod wifi;
}
