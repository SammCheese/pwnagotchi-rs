#![warn(
  clippy::complexity,
  clippy::style,
  clippy::suspicious,
  clippy::pedantic,
  clippy::nursery,
  clippy::cargo
)]
#![allow(clippy::multiple_crate_versions)]

pub mod ui {
  pub mod components;
  pub mod draw;
  pub mod fonts;
  pub mod refresher;
  pub mod state;
  pub mod view;
}

pub mod web {
  pub mod frame;
  pub mod server;
  pub mod pages {
    pub mod context;
    pub mod handler;
    pub mod routes;
  }
}
