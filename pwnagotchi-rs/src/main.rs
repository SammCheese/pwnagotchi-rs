extern crate pwnagotchi_rs;

use std::{process::exit, sync::Arc};

use clap::Parser;
use nix::libc::EXIT_SUCCESS;
use parking_lot::RwLock;
use pwnagotchi_core::{
  agent::{Agent, AgentComponent},
  automata::{Automata, AutomataComponent},
  bettercap::{Bettercap, BettercapComponent},
  cli::CliComponent,
  events::eventlistener::EventListenerComponent,
  mesh::advertiser::AdvertiserComponent,
  setup::SetupComponent,
};
use pwnagotchi_rs::components::manager::ComponentManager;
use pwnagotchi_shared::{
  config::{config, init_config},
  identity::{Identity, IdentityComponent},
  logger::LOGGER,
  sessions::manager::SessionManager,
  traits::{
    agent::AgentTrait,
    automata::AutomataTrait,
    bettercap::BettercapTrait,
    epoch::Epoch,
    general::{Component, CoreModules},
    ui::ViewTrait,
  },
};
use pwnagotchi_ui::{
  ui::{
    refresher::RefresherComponent,
    view::{View, ViewComponent},
  },
  web::server::ServerComponent,
};

#[derive(Parser, Debug)]
struct CliArgs {
  #[clap(
    short = 'C',
    long = "config",
    default_value = "/etc/pwnagotchi/config.toml",
    help = "The configuration file to use"
  )]
  config: String,
  #[clap(
    short = 'D',
    long = "device",
    default_value = "dev",
    help = "Start Pwnagotchi in the specified mode (dev, pi, portable)"
  )]
  device: String,
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
async fn main() -> anyhow::Result<()> {
  let cli = CliArgs::parse();

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

  let identity = Arc::new(RwLock::new(Identity::new()));
  let session_manager = Arc::new(SessionManager::new());
  let epoch = Arc::new(RwLock::new(Epoch::new()));
  let bettercap = Arc::new(Bettercap::new()) as Arc<dyn BettercapTrait + Send + Sync>;

  let view = Arc::new(View::new(Arc::clone(&epoch))) as Arc<dyn ViewTrait + Send + Sync>;
  let automata = Arc::new(Automata::new(
    Arc::clone(&epoch),
    Arc::clone(&view) as Arc<dyn ViewTrait + Send + Sync>,
  )) as Arc<dyn AutomataTrait + Send + Sync>;

  let agent = Arc::new(Agent::new(
    Arc::clone(&automata),
    Arc::clone(&bettercap),
    Arc::clone(&epoch),
    Arc::clone(&view),
    Arc::clone(&session_manager),
  )) as Arc<dyn AgentTrait + Send + Sync>;

  let core_modules = Arc::new(CoreModules {
    session_manager: Arc::clone(&session_manager),
    identity: Arc::clone(&identity),
    epoch: Arc::clone(&epoch),
    bettercap: Arc::clone(&bettercap),
    view: Arc::clone(&view),
    agent: Arc::clone(&agent),
    automata: Arc::clone(&automata),
  });

  LOGGER.log_debug("Pwnagotchi", "Loading Components");

  let mut manager = ComponentManager::new(Arc::clone(&core_modules));

  let components: Vec<Box<dyn Component + Send + Sync>> = vec![
    Box::new(IdentityComponent::new()),
    Box::new(BettercapComponent::new()),
    Box::new(EventListenerComponent::new()),
    Box::new(ViewComponent::new()),
    Box::new(AgentComponent::new()),
    Box::new(AutomataComponent::new()),
    Box::new(ServerComponent::new()),
    Box::new(AdvertiserComponent::new()),
    Box::new(RefresherComponent::new()),
    Box::new(SetupComponent::new()),
  ];

  for component in components {
    manager.register(component);
  }

  if let Err(e) = manager.init_all().await {
    eprintln!("Failed to initialize components: {}", e);
    exit(1);
  }

  if let Err(e) = manager.start_all().await {
    eprintln!("Failed to start components: {}", e);
  }

  // Cli Routines have to go last ALWAYS
  let mut cli = CliComponent::new(cli.manual);
  if let Err(e) = cli.init(&Arc::clone(&core_modules)).await {
    eprintln!("Failed to initialize CLI component: {}", e);
    exit(1);
  }
  if let Err(e) = cli.start().await {
    eprintln!("Failed to start CLI component: {}", e);
    exit(1);
  }

  tokio::signal::ctrl_c().await?;

  LOGGER.log_info("Pwnagotchi", "Shutting down...");

  manager.shutdown().await;
  Ok(())
}

pub const fn version() -> &'static str {
  env!("CARGO_PKG_VERSION")
}
