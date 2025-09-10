extern crate pwnagotchi_rs;

use std::{process::exit, sync::Arc};

use clap::Parser;
use nix::libc::EXIT_SUCCESS;
use parking_lot::Mutex;
use pwnagotchi_core::{
  agent::Agent,
  ai::Epoch,
  automata::Automata,
  bettercap::{Bettercap, spawn_bettercap},
  cli,
  events::eventlistener::start_event_loop,
  mesh::advertiser::{AsyncAdvertiser, start_advertising},
  traits::bettercapcontroller::BettercapController,
  voice::Voice,
};
use pwnagotchi_hw::display::base::get_display_from_config;
use pwnagotchi_shared::{
  config::{config, init_config},
  identity::Identity,
  logger::LOGGER,
  sessions::manager::SessionManager,
  traits::{automata::AgentObserver, ui::ViewTrait, voice::VoiceTrait},
};
use pwnagotchi_ui::{
  ui::{refresher::start_sessionfetcher, view::View},
  web::server::Server,
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

pub struct PwnContext {
  pub agent: Arc<Agent>,
  pub session_manager: Arc<SessionManager>,
  pub view: Arc<dyn ViewTrait + Send + Sync>,
  pub epoch: Arc<Mutex<Epoch>>,
  pub identity: Arc<Identity>,
  pub bettercap: Arc<Bettercap>,
  pub bc_controller: Arc<dyn BettercapController>,
  pub automata: Arc<dyn AgentObserver + Send + Sync>,
  pub advertiser: Arc<Mutex<AsyncAdvertiser>>,
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

  let ctx = Arc::new(build_context().await);

  // Render Changes to UI
  let ctx_clone = Arc::clone(&ctx);
  tokio::task::spawn(async move {
    ctx_clone.view.start_render_loop().await;
  });

  // Fetch Session data for UI
  let ctx_clone = Arc::clone(&ctx);
  tokio::task::spawn(async move {
    start_sessionfetcher(&ctx_clone.session_manager, &ctx_clone.view).await;
  });

  // Bettercap Event Websocket
  let ctx_clone = Arc::clone(&ctx);
  let bettercap = Arc::clone(&ctx_clone.bettercap);
  tokio::task::spawn(async move {
    bettercap.run_websocket().await;
  });

  // Event Listener
  let ctx_clone = Arc::clone(&ctx);
  tokio::task::spawn(async move {
    start_event_loop(
      &ctx_clone.session_manager,
      &ctx_clone.bc_controller,
      &ctx_clone.epoch,
      &ctx_clone.view,
    )
    .await;
  });

  // WEB UI
  let ctx_clone = Arc::clone(&ctx);
  tokio::task::spawn(async move {
    Server::new().start(&ctx_clone.session_manager, &ctx_clone.identity);
  });

  // Advertiser
  let ctx_clone = Arc::clone(&ctx);
  tokio::task::spawn(async move {
    start_advertising(&ctx_clone.advertiser, &ctx_clone.session_manager, &ctx_clone.view).await;
  });

  let ctx_clone = Arc::clone(&ctx);
  LOGGER.log_info(
    "Pwnagotchi",
    &format!(
      "Pwnagotchi {}@{} (v{})",
      config().main.name,
      ctx_clone.identity.fingerprint(),
      env!("CARGO_PKG_VERSION")
    ),
  );

  if cli.manual {
    cli::do_manual_mode(&ctx_clone.session_manager, &ctx_clone.agent).await;
  } else {
    cli::do_auto_mode(
      &ctx_clone.session_manager,
      &ctx_clone.bc_controller,
      &ctx_clone.agent,
      &ctx_clone.automata,
    )
    .await;
  };
}

async fn build_context() -> PwnContext {
  let identity = Arc::new(Identity::new());
  let epoch = Arc::new(parking_lot::Mutex::new(Epoch::new()));
  let voice: Arc<dyn VoiceTrait + Send + Sync> = Arc::new(Voice::new());
  let view: Arc<dyn ViewTrait + Send + Sync> =
    Arc::new(View::new(&get_display_from_config(), &voice));
  let sm = Arc::new(SessionManager::new(&view));

  // PERSONALITY
  let automata: Arc<dyn AgentObserver + Send + Sync> =
    Arc::new(Automata::new(Arc::clone(&epoch), Arc::clone(&view)));
  let auto_handle = Arc::clone(&automata);

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

  PwnContext {
    agent,
    session_manager: sm,
    view,
    epoch,
    identity,
    bettercap,
    bc_controller,
    automata,
    advertiser: adv,
  }
}

pub const fn version() -> &'static str {
  env!("CARGO_PKG_VERSION")
}
