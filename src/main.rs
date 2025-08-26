#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::todo)]
#![warn(clippy::panic)]
#![warn(clippy::clone_on_copy)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::needless_collect)]
#![warn(clippy::single_match)]
#![warn(clippy::wildcard_imports)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::struct_excessive_bools)]

extern crate pwnagotchi_rs;

use std::{ process::exit, sync::Arc };
use clap::Parser;
use nix::libc::{ EXIT_SUCCESS };
use pwnagotchi_rs::core::{
  agent::Agent,
  cli,
  config::{ config, init_config },
  events::{listener::EventListener},
};
use tokio::{sync::Mutex};

#[derive(Parser, Debug)]
struct Cli {
  #[clap(
    short = 'C',
    long = "config",
    default_value = "/etc/pwnagotchi/config.toml",
    help = "The configuration file to use"
  )]
  config: String,
  #[clap(short = 'l', long = "log-level", default_value = "info", help = "The log level to use")]
  log_level: String,
  #[clap(short = 'm', long = "manual", default_value = "false", help = "Whether to do manual mode")]
  manual: bool,
  #[clap(short, long = "clear", default_value = "false", help = "Clears the screen and exits")]
  clear: bool,
  #[clap(short, long = "debug", default_value = "false", help = "Enables debug mode")]
  debug: bool,
  #[clap(long = "version", help = "Prints the version information")]
  show_version: bool,
  #[clap(long = "print-config", help = "Prints the configuration")]
  print_config: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

  if cli.clear {
    println!("\x1B[2J\x1B[1;1H");
    exit(EXIT_SUCCESS);
  }
  if cli.show_version {
    println!("Version: {}", version());
    exit(EXIT_SUCCESS);
  }

  // will default to /etc/pwnagotchi/config.toml unless specified
  init_config(&cli.config);

  if cli.print_config {
    println!("Configuration: {:?}", config());
    exit(EXIT_SUCCESS);
  }

  let agent = Arc::new(Mutex::new(Agent::new()));
  let _e = EventListener::new(Arc::clone(&agent)).start_event_loop();

  if cli.manual {
    cli::do_manual_mode(agent);
  } else {
    cli::do_auto_mode(agent).await;
  }

}

#[must_use]
pub const fn version() -> &'static str {
  env!("CARGO_PKG_VERSION")
}
