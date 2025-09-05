#![warn(
  clippy::complexity,
  clippy::style,
  clippy::suspicious,
  clippy::pedantic,
  clippy::nursery,
  clippy::cargo
)]
#![deny(clippy::correctness, clippy::perf)]
// I occasionally add functions required but not implemented yet
// Helps with TODOS
#![allow(dead_code, reason = "Occasional placeholders for future implementation")]
// I hate documenting
#![allow(
  clippy::missing_errors_doc,
  clippy::missing_docs_in_private_items,
  reason = "Documentation will be added later as the project matures:tm:"
)]
#![allow(clippy::must_use_candidate)]
// Cant do much about that
#![allow(clippy::multiple_crate_versions)]

pub mod core {
  pub mod agent;
  pub mod ai;
  pub mod automata;
  pub mod bettercap;
  pub mod cli;
  pub mod config;
  pub mod identity;
  pub mod log;
  pub mod mesh;
  pub mod models;
  pub mod setup;
  pub mod utils;
  pub mod voice;

  pub mod ui {
    pub mod components;
    pub mod draw;
    pub mod fonts;
    pub mod old;
    pub mod refresher;
    pub mod state;
    pub mod view;
  }

  pub mod sessions {
    pub mod lastsession;
    pub mod manager;
    pub mod session;
  }

  pub mod events {
    pub mod eventlistener;
  }

  pub mod traits {
    pub mod agentobserver;
    pub mod bettercapcontroller;

    pub mod hostname;
    pub mod logger;
    pub mod syscontrol;
    pub mod sysdata;
  }
}

mod net {}
