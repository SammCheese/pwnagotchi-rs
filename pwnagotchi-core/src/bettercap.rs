use std::{
  borrow::Cow,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  time::Duration,
};

use anyhow::Result;
use base64::{Engine, engine::general_purpose};
use futures_util::{SinkExt, StreamExt};
use pwnagotchi_shared::{
  config::config,
  logger::LOGGER,
  models::bettercap::BettercapSession,
  traits::{
    bettercap::{BettercapCommand, BettercapTrait},
    general::{Component, CoreModule, CoreModules, Dependencies},
  },
};
use tokio::{sync::broadcast, task::JoinHandle};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use ureq::{
  Agent, Body, SendBody,
  http::{Request, Response, header::HeaderValue},
  middleware::MiddlewareNext,
};

#[async_trait::async_trait]
impl BettercapTrait for Bettercap {
  async fn send(&self, cmd: BettercapCommand) -> anyhow::Result<()> {
    match cmd {
      BettercapCommand::Run { cmd, respond_to } => {
        let res = self.run(&cmd).await;
        let _ = respond_to.send(res);
      }
      BettercapCommand::GetSession { respond_to } => {
        let res = self.session(None);
        let _ = respond_to.send(res);
      }
      BettercapCommand::SubscribeEvents { respond_to } => {
        let rx = self.subscribe_events();
        let _ = respond_to.send(rx);
      }
    }
    Ok(())
  }

  async fn session(&self) -> anyhow::Result<Option<BettercapSession>> {
    Ok(self.session(None))
  }

  async fn run_websocket(&self) {
    Bettercap::run_websocket(self).await;
  }

  fn is_ready(&self) -> bool {
    Bettercap::is_ready(self)
  }

  async fn run(&self, cmd: &str) -> Result<(), anyhow::Error> {
    Bettercap::run(self, cmd).await
  }
}

pub struct BettercapComponent {
  bettercap: Option<Arc<dyn BettercapTrait + Send + Sync>>,
}

impl Default for BettercapComponent {
  fn default() -> Self {
    Self::new()
  }
}

impl BettercapComponent {
  pub fn new() -> Self {
    Self { bettercap: None }
  }
}

impl Dependencies for BettercapComponent {
  fn name(&self) -> &'static str {
    "BettercapComponent"
  }

  fn dependencies(&self) -> &[&str] {
    &["Bettercap"]
  }
}

#[async_trait::async_trait]
impl Component for BettercapComponent {
  async fn init(&mut self, ctx: &CoreModules) -> Result<()> {
    self.bettercap = Some(Arc::clone(&ctx.bettercap));
    Ok(())
  }

  async fn start(&self) -> Result<Option<JoinHandle<()>>> {
    if let Some(bc) = &self.bettercap
      && !bc.is_ready()
    {
      LOGGER.log_info("Bettercap", "Starting Bettercap WebSocket connection...");
      let bc = Arc::clone(bc);
      let handle = tokio::spawn(async move {
        bc.run_websocket().await;
      });

      return Ok(Some(handle));
    }
    Ok(None)
  }
}

#[derive(Debug, Clone)]
pub struct Bettercap {
  pub retries: u32,
  pub ping_timeout: u64,
  pub ping_interval: u64,
  pub max_queue: usize,
  pub min_sleep: f64,
  pub max_sleep: f64,
  pub is_ready: Arc<AtomicBool>,

  hostname: Cow<'static, str>,
  port: u16,
  username: Cow<'static, str>,
  password: Cow<'static, str>,
  url: String,
  websocket_url: String,
  scheme: Cow<'static, str>,

  event_tx: broadcast::Sender<String>,
  req_client: Agent,
}

impl CoreModule for Bettercap {
  fn name(&self) -> &'static str {
    "Bettercap"
  }
}

impl Default for Bettercap {
  fn default() -> Self {
    Self::new()
  }
}

fn bettercap_add_authorization(
  mut req: Request<SendBody>,
  next: MiddlewareNext,
) -> Result<Response<Body>, ureq::Error> {
  let username = config().bettercap.username.to_string();
  let password = config().bettercap.password.to_string();
  req.headers_mut().insert(
    "Authorization",
    HeaderValue::from_str(&format!(
      "Basic {}",
      general_purpose::STANDARD.encode(format!("{username}:{password}"))
    ))
    .unwrap(),
  );
  next.handle(req)
}

impl Bettercap {
  pub fn new() -> Self {
    let scheme = if config().bettercap.port == 443 { "https" } else { "http" };
    let ws_scheme = if config().bettercap.port == 443 { "wss" } else { "ws" };
    let max_queue = 10_000usize;
    let (event_tx, _rx) = broadcast::channel(max_queue);

    let agent_config = ureq::Agent::config_builder()
      .timeout_global(Some(std::time::Duration::from_secs(5)))
      .middleware(bettercap_add_authorization)
      .build();
    let req_client = Agent::new_with_config(agent_config);

    Self {
      retries: 5,
      ping_timeout: 180,
      ping_interval: 15,
      max_queue,
      min_sleep: 0.5,
      max_sleep: 5.0,
      hostname: Cow::Borrowed(&config().bettercap.hostname),
      port: config().bettercap.port,
      username: Cow::Borrowed(&config().bettercap.username),
      password: Cow::Borrowed(&config().bettercap.password),
      url: format!(
        "{}://{}:{}@{}:{}/api",
        scheme,
        config().bettercap.username,
        config().bettercap.password,
        config().bettercap.hostname,
        config().bettercap.port
      ),
      websocket_url: format!(
        "{}://{}:{}@{}:{}/api/events",
        ws_scheme,
        config().bettercap.username,
        config().bettercap.password,
        config().bettercap.hostname,
        config().bettercap.port
      ),
      scheme: Cow::Borrowed(scheme),
      is_ready: Arc::new(AtomicBool::new(false)),
      event_tx,
      req_client,
    }
  }

  pub fn is_ready(&self) -> bool {
    self.is_ready.load(Ordering::SeqCst)
  }

  pub fn session(&self, sess: Option<&str>) -> Option<BettercapSession> {
    let sess = sess.unwrap_or("session");
    let base = self
      .url
      .replace("%{scheme}", &self.scheme)
      .replace("%{username}", &self.username)
      .replace("%{password}", &self.password)
      .replace("%{hostname}", &self.hostname)
      .replace("%{port}", &self.port.to_string());

    let url = format!("{base}/{sess}");
    let cli = self.req_client.get(url).call();

    cli.map_or(None, |mut resp| match resp.body_mut().read_json::<BettercapSession>() {
      Ok(session) => Some(session),
      Err(e) => {
        LOGGER.log_error("Bettercap", &format!("Failed to parse Bettercap session JSON: {e}"));
        None
      }
    })
  }

  pub async fn run_websocket(&self) {
    let ws_url = self.websocket_url.clone();
    let min_sleep = self.min_sleep;
    let max_sleep = self.max_sleep;

    LOGGER.log_info("Bettercap", "Connecting to Event WebSocket");

    loop {
      match connect_async(&ws_url).await {
        Ok((ws_stream, _)) => {
          self.is_ready.store(true, Ordering::SeqCst);
          LOGGER.log_info("Bettercap", "Event WebSocket connected");
          let (mut write, mut read) = ws_stream.split();
          while let Some(msg) = read.next().await {
            match msg {
              Ok(Message::Text(txt)) => {
                let _ = self.event_tx.send(txt.to_string());
              }
              Ok(Message::Binary(_) | Message::Ping(_) | Message::Pong(_) | Message::Frame(_)) => {}
              Ok(Message::Close(_)) => {
                LOGGER.log_warning("Bettercap", "Event WebSocket closed by server");
                self.is_ready.store(false, Ordering::SeqCst);
                break;
              }
              Err(_e) => {
                LOGGER.log_warning("Bettercap", "Lost Event Websocket, attempting to reconnect...");
                self.is_ready.store(false, Ordering::SeqCst);
                // Still there?
                if (write.send(Message::Ping(vec![].into())).await).is_err() {
                  LOGGER.log_warning("Bettercap", "Ping to Event Websocket failed, this is bad...");
                  break;
                }
                LOGGER.log_warning("Bettercap", "Ping OK, keeping connection alive...");
              }
            }
          }
        }
        Err(_e) => {}
      }
      let sleep_time = (max_sleep - min_sleep).mul_add(fastrand::f64(), min_sleep);
      LOGGER.log_debug("Bettercap", &format!("Reconnecting in {sleep_time:.1} seconds"));
      tokio::time::sleep(Duration::from_secs_f64(sleep_time)).await;
    }
  }

  pub fn subscribe_events(&self) -> broadcast::Receiver<String> {
    self.event_tx.subscribe()
  }

  pub async fn run(&self, cmd: &str) -> Result<(), anyhow::Error> {
    let url = self
      .url
      .replace("%{scheme}", &self.scheme)
      .replace("%{username}", &self.username)
      .replace("%{password}", &self.password)
      .replace("%{hostname}", &self.hostname)
      .replace("%{port}", &self.port.to_string())
      + "/session".trim_end_matches('/');

    let mut retries_left = self.retries;

    loop {
      LOGGER.log_debug("Bettercap", &format!("Commanding Bettercap to {cmd}"));
      let agent = self
        .req_client
        .post(&url)
        .config()
        .timeout_global(Some(Duration::from_secs(2)))
        .build();
      let req = agent.send_json(serde_json::json!({"cmd": cmd}));

      match req {
        Ok(resp) => {
          // Bad request could come from an already existing session + setup
          if resp.status().is_success() || resp.status() == 400 {
            return Ok(());
          }
          LOGGER.log_error("Bettercap", &format!("Request failed with status {}", resp.status()));
        }
        Err(ureq::Error::StatusCode(400..410) | ureq::Error::Timeout(_)) => {
          // The server clearly got the request but didnt like it
          // Dont try this again.
          return Ok(());
        }
        Err(e) => {
          LOGGER.log_warning("Bettercap", &format!("Request error: {e}"));
        }
      }
      if retries_left == 0 {
        return Err(anyhow::anyhow!("Request failed"));
      }
      retries_left -= 1;
      tokio::time::sleep(Duration::from_secs(self.ping_interval)).await;
    }
  }
}
