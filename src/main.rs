extern crate pwnagotchi_rs;

use std::{process::exit, sync::Arc};

use clap::Parser;
use nix::libc::EXIT_SUCCESS;
use parking_lot::Mutex;
use pwnagotchi_rs::core::{
  agent::Agent,
  ai::Epoch,
  automata::Automata,
  bettercap::{Bettercap, spawn_bettercap},
  cli,
  config::{config, init_config},
  events::eventlistener::start_event_loop,
  identity::Identity,
  log::LOGGER,
  mesh::advertiser::{AsyncAdvertiser, start_advertising},
  sessions::manager::SessionManager,
  traits::bettercapcontroller::BettercapController,
  ui::{
    old::{hw::base::get_display_from_config, web::server::Server},
    refresher::start_sessionfetcher,
    view::View,
  },
};

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
  #[clap(
    short = 'm',
    long = "manual",
    default_value = "false",
    help = "Whether to do manual mode"
  )]
  manual: bool,
  #[clap(short, long = "clear", default_value = "false", help = "Clears the screen and exits")]
  clear: bool,
  #[clap(short, long = "debug", default_value = "false", help = "Enables debug mode")]
  debug: bool,
  #[clap(long = "version", help = "Prints the version information")]
  show_version: bool,
  #[clap(long = "print-config", help = "Prints the configuration")]
  print_config: bool,
  #[clap(long = "skip", help = "Skip parsing")]
  skip: bool,
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

  // Must complete before anything else
  let identity = Identity::new();
  let sm = Arc::new(SessionManager::new());

  let epoch = Arc::new(parking_lot::Mutex::new(Epoch::new()));
  let view = Arc::new(View::new(&get_display_from_config()));

  // PERSONALITY
  let automata = Automata::new(Arc::clone(&epoch), Arc::clone(&view));
  let auto_handle = Arc::new(automata);

  // BETTERCAP
  let bettercap = Arc::new(Bettercap::new());
  let bc_controller: Arc<dyn BettercapController> = Arc::new(spawn_bettercap(&bettercap));

  // AGENT
  let agent_bc = Arc::clone(&bc_controller);
  let agent = Arc::new(Agent::new(&auto_handle, &agent_bc, &epoch, &view));

  // ADVERTISER
  let advertiser_view = Arc::clone(&view);
  let adv = Arc::new(Mutex::new(AsyncAdvertiser::new(
    Arc::clone(&epoch),
    &identity,
    Some(advertiser_view),
  )));

  // Render Changes to UI
  let view_clone = Arc::clone(&view);
  tokio::task::spawn(async move {
    view_clone.start_render_loop().await;
  });

  // Fetch Session data for UI
  let sm1 = Arc::clone(&sm);
  let view1 = Arc::clone(&view);
  tokio::task::spawn(async move {
    start_sessionfetcher(&sm1, &view1).await;
  });

  // Bettercap Event Websocket
  let bettercap = Arc::clone(&bettercap);
  tokio::task::spawn(async move {
    bettercap.run_websocket().await;
  });

  // Event Listener
  let view2 = Arc::clone(&view);
  let sm2 = Arc::clone(&sm);
  tokio::task::spawn(async move {
    start_event_loop(&sm2, &bc_controller, &epoch, &view2).await;
  });

  // WEB UI
  tokio::task::spawn(async move {
    Server::new().start();
  });

  // Advertiser
  let aadv = Arc::clone(&adv);
  let asm = Arc::clone(&sm);
  let aview = Arc::clone(&view);
  tokio::task::spawn(async move {
    start_advertising(&aadv, &asm, &aview).await;
  });

  LOGGER.log_info(
    "Pwnagotchi",
    &format!(
      "Pwnagotchi {}@{} (v{})",
      config().main.name,
      identity.fingerprint(),
      env!("CARGO_PKG_VERSION")
    ),
  );

  if cli.manual {
    cli::do_manual_mode(&sm, &agent).await;
  } else {
    cli::do_auto_mode(&sm, &agent_bc, &agent, Arc::clone(&auto_handle)).await;
  };
}

pub const fn version() -> &'static str {
  env!("CARGO_PKG_VERSION")
}
