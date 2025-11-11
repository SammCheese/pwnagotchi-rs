extern crate pwnagotchi_rs;

use std::{process::exit, sync::Arc};

use clap::Parser;
use nix::libc::EXIT_SUCCESS;
use parking_lot::RwLock;
use pwnagotchi_core::{
  agent::{Agent, AgentComponent},
  automata::{Automata, AutomataComponent},
  bettercap::{Bettercap, BettercapComponent},
  cli::Cli,
  events::eventlistener::EventListenerComponent,
  grid::Grid,
  mesh::advertiser::AdvertiserComponent,
  setup::SetupComponent,
};
use pwnagotchi_plugins::managers::plugin_manager::PluginManager;
use pwnagotchi_rs::components::manager::ComponentManager;
use pwnagotchi_shared::{
  config::{config_read, init_config},
  identity::{Identity, IdentityComponent},
  logger::LOGGER,
  sessions::manager::SessionManager,
  traits::{
    agent::AgentTrait,
    automata::AutomataTrait,
    bettercap::BettercapTrait,
    epoch::Epoch,
    events::EventBus,
    general::{Component, CoreModules},
    grid::GridTrait,
    ui::ViewTrait,
  },
  types::events::EventPayload,
};
use pwnagotchi_ui::{
  ui::{
    refresher::RefresherComponent,
    view::{View, ViewComponent},
  },
  web::server::{Server, build_router},
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
    println!("Configuration: {:?}", config_read());
    exit(EXIT_SUCCESS);
  }

  // Create Managers
  let plugin_manager_inner = PluginManager::new();
  let event_bus = plugin_manager_inner.event_bus();
  let plugin_manager = Arc::new(RwLock::new(plugin_manager_inner));
  let mut component_manager = ComponentManager::new();

  // Build CoreModules
  let core_modules = build_coremodules(event_bus.clone());

  // Set CoreModules for Components
  component_manager.set_core_modules(Arc::clone(&core_modules));

  // Initialize Plugins
  plugin_manager.write().init();
  plugin_manager.write().set_core_modules(Arc::clone(&core_modules));
  plugin_manager.write().load_plugins();
  plugin_manager.write().initialize_plugins();

  // Immediately emit starting plugin event
  let event = Arc::clone(&core_modules.events);
  tokio::task::spawn(async move {
    let _ = event.emit_payload("starting", EventPayload::empty()).await;
  });

  // Build Router and Start WebServer
  let router = build_router(
    Arc::clone(&core_modules.session_manager),
    Arc::clone(&core_modules.identity),
    Arc::clone(&plugin_manager),
    Arc::clone(&core_modules.grid),
  );
  tokio::task::spawn(async move {
    let _ = Server::new(router).start_server().await;
  });

  LOGGER.log_debug("Pwnagotchi", "Loading Components");

  let components: Vec<Box<dyn Component + Send + Sync>> = vec![
    Box::new(IdentityComponent::new()),
    Box::new(BettercapComponent::new()),
    Box::new(EventListenerComponent::new()),
    Box::new(ViewComponent::new()),
    Box::new(AgentComponent::new()),
    Box::new(AutomataComponent::new()),
    Box::new(AdvertiserComponent::new()),
    Box::new(RefresherComponent::new()),
    Box::new(SetupComponent::new()),
  ];

  for component in components {
    component_manager.register(component);
  }

  let _ = component_manager.init_all().await;
  let _ = component_manager.start_all().await;

  let plug_manager = Arc::clone(&plugin_manager);

  // Cli Routines have to go last ALWAYS
  let controller = Cli::new(Arc::clone(&core_modules));

  tokio::task::spawn(async move {
    if cli.manual {
      controller.do_manual_mode().await;
    } else {
      controller.do_auto_mode().await;
    }
  });

  tokio::select! {
    _ = tokio::signal::ctrl_c() => {
      LOGGER.log_info("Pwnagotchi", "Shutting down...");
      let _ = plug_manager.write().shutdown_all();
      component_manager.shutdown().await;
    },
  }
  Ok(())
}

fn build_coremodules(events: Arc<dyn EventBus>) -> Arc<CoreModules> {
  let identity = Arc::new(RwLock::new(Identity::new()));
  let session_manager = Arc::new(SessionManager::new());
  let epoch = Arc::new(RwLock::new(Epoch::new()));
  let bettercap = Arc::new(Bettercap::new()) as Arc<dyn BettercapTrait + Send + Sync>;

  let view = Arc::new(View::new(Arc::clone(&epoch))) as Arc<dyn ViewTrait + Send + Sync>;
  let automata = Arc::new(Automata::new(
    Arc::clone(&epoch),
    Arc::clone(&events),
    Arc::clone(&view) as Arc<dyn ViewTrait + Send + Sync>,
  )) as Arc<dyn AutomataTrait + Send + Sync>;

  let agent = Arc::new(Agent::new(
    Arc::clone(&automata),
    Arc::clone(&bettercap),
    Arc::clone(&epoch),
    Arc::clone(&view),
    Arc::clone(&session_manager),
  )) as Arc<dyn AgentTrait + Send + Sync>;

  let grid = Arc::new(Grid::new()) as Arc<dyn GridTrait + Send + Sync>;

  Arc::new(CoreModules {
    session_manager: Arc::clone(&session_manager),
    identity: Arc::clone(&identity),
    epoch: Arc::clone(&epoch),
    bettercap: Arc::clone(&bettercap),
    view: Arc::clone(&view),
    agent: Arc::clone(&agent),
    automata: Arc::clone(&automata),
    grid: Arc::clone(&grid),
    events,
  })
}

pub const fn version() -> &'static str {
  env!("CARGO_PKG_VERSION")
}
