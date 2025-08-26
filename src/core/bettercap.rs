use crate::core::models::bettercap::BettercapSession;
use crate::core::config::{ config };
use crate::core::log::LOGGER;
use std::{ sync::Arc, time::Duration };
use std::sync::atomic::{ AtomicBool, Ordering };
use tokio::{ net::TcpStream };
use tokio_tungstenite::{ connect_async, MaybeTlsStream, WebSocketStream };
use tungstenite::protocol::Message;
use futures_util::{ stream::{ SplitStream, StreamExt }, SinkExt };
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct Bettercap {
  pub bettercap_path: String,
  pub retries: u32,
  pub ping_timeout: u64,
  pub ping_interval: u64,
  pub max_queue: usize,
  pub min_sleep: f64,
  pub max_sleep: f64,
  pub is_ready: Arc<AtomicBool>,

  hostname: String,
  port: u16,
  username: String,
  password: String,
  url: String,
  websocket_url: String,
  scheme: String,

  // Broadcast channel for websocket events
  event_tx: broadcast::Sender<String>,
}

impl Default for Bettercap {
  fn default() -> Self {
    let (event_tx, _rx) = broadcast::channel(10_000);
    Self {
      bettercap_path: "/usr/bin/bettercap".into(),
      retries: 5,
      ping_timeout: 180,
      ping_interval: 15,
      max_queue: 10000,
      min_sleep: 0.5,
      max_sleep: 5.0,
      hostname: "127.0.0.1".into(),
      scheme: "http".into(),
      port: 8081,
      username: "user".into(),
      password: "pass".into(),
      url: "%{scheme}://%{username}:%{password}@%{hostname}:%{port}/api".into(),
      websocket_url: "ws://%{username}:%{password}@%{hostname}:%{port}/api/events".into(),
      is_ready: Arc::new(AtomicBool::new(false)),
      event_tx,
    }
  }
}

type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

impl Bettercap {
  pub fn new() -> Self {
    let scheme = if config().bettercap.port == 443 { "https" } else { "http" };
    let ws_scheme = if config().bettercap.port == 443 { "wss" } else { "ws" };
    let max_queue = 10_000usize;
    let (event_tx, _rx) = broadcast::channel(max_queue);
    Self {
      bettercap_path: config().main.bettercap_path.clone(),
      retries: 5,
      ping_timeout: 180,
      ping_interval: 15,
      max_queue,
      min_sleep: 0.5,
      max_sleep: 5.0,
      hostname: config().bettercap.hostname.clone(),
      port: config().bettercap.port,
      username: config().bettercap.username.clone(),
      password: config().bettercap.password.clone(),
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
      scheme: scheme.to_string(),
      is_ready: Arc::new(AtomicBool::new(false)),
      event_tx,
    }
  }

  #[must_use]
  pub fn is_ready(&self) -> bool {
    self.is_ready.load(Ordering::SeqCst)
  }

  pub async fn session(&self, sess: Option<&str>) -> Option<BettercapSession> {
    let sess = sess.unwrap_or("session");

    let base = self.url
      .replace("%{scheme}", &self.scheme)
      .replace("%{username}", &self.username)
      .replace("%{password}", &self.password)
      .replace("%{hostname}", &self.hostname)
      .replace("%{port}", &self.port.to_string());

    let url = format!("{base}/{sess}");
    let client = reqwest::Client::new();
    let cli = client.get(url).basic_auth(&self.username, Some(&self.password)).send().await;

    match cli {
      Ok(resp) => {
        match resp.json::<BettercapSession>().await {
          Ok(session) => { Some(session) }
          Err(e) => {
            LOGGER.log_error("Bettercap", &format!("Failed to parse session JSON: {e}"));
            None
          }
        }
      }
      Err(e) => {
        LOGGER.log_error("Bettercap", &format!("HTTP request failed: {e}"));
        None
      }
    }
  }

  pub async fn run_websocket(&self) {
    let ws_url = self.websocket_url.clone();
    let min_sleep = self.min_sleep;
    let max_sleep = self.max_sleep;
    loop {
      LOGGER.log_info("Bettercap", &format!("Connecting websocket to {ws_url}"));
      match connect_async(&ws_url).await {
        Ok((ws_stream, _)) => {
          self.is_ready.store(true, Ordering::SeqCst);
          LOGGER.log_info("Bettercap", "WebSocket connected");
          let (mut write, mut read) = ws_stream.split();
          while let Some(msg) = read.next().await {
            match msg {
              Ok(Message::Text(txt)) => {
                let _ = self.event_tx.send(txt.to_string());
              }
              Ok(Message::Binary(_)) => {
                LOGGER.log_debug("Bettercap", "Ignoring binary WS frame");
              }
              Ok(Message::Ping(_) | Message::Pong(_) | Message::Frame(_)) => {}
              Ok(Message::Close(_)) => {
                LOGGER.log_warning("Bettercap", "WebSocket closed by server");
                self.is_ready.store(false, Ordering::SeqCst);
                break;
              }
              Err(e) => {
                LOGGER.log_error("Bettercap", &format!("WebSocket error: {e}"));
                self.is_ready.store(false, Ordering::SeqCst);
                // Still there?
                if let Err(ping_err) = write.send(Message::Ping(vec![].into())).await {
                  LOGGER.log_warning(
                    "Bettercap",
                    &format!("Ping failed: {ping_err}, reconnecting...")
                  );
                  break;
                }
                LOGGER.log_warning("Bettercap", "Ping OK, keeping connection alive...");
              }
            }
          }
        }
        Err(e) => {
          LOGGER.log_error("Bettercap", &format!("WebSocket connect failed: {e}"));
        }
      }
      let sleep_time = (max_sleep - min_sleep).mul_add(rand::random::<f64>(), min_sleep);
      LOGGER.log_info("Bettercap", &format!("Reconnecting in {sleep_time:.1} seconds"));
      tokio::time::sleep(Duration::from_secs_f64(sleep_time)).await;
    }
  }

  pub fn subscribe_events(&self) -> broadcast::Receiver<String> {
    self.event_tx.subscribe()
  }

  /// Sends a command to Bettercap via HTTP POST.
  ///
  /// # Arguments
  ///
  /// * `args` - A slice of command arguments to send.
  ///
  /// # Errors
  ///
  /// Returns an error if the HTTP request fails or if Bettercap does not respond
  pub async fn run(&self, args: &[&str]) -> Result<(), anyhow::Error> {
    let url =
      self.url
        .replace("%{scheme}", &self.scheme)
        .replace("%{username}", &self.username)
        .replace("%{password}", &self.password)
        .replace("%{hostname}", &self.hostname)
        .replace("%{port}", &self.port.to_string()) + "/session".trim_end_matches('/');

    let mut retries_left = self.retries;
    let client = reqwest::Client::new();
    let cmd: String = args.join(" ");

    loop {
      LOGGER.log_debug("Bettercap", &format!("Commanding Bettercap to {cmd}"));
      LOGGER.log_debug("Bettercap", &format!("{}", &serde_json::json!({"cmd": cmd})));
      let req = client
        .post(&url)
        .json(&serde_json::json!({"cmd": cmd}))
        .basic_auth(&self.username, Some(&self.password))
        .timeout(Duration::from_secs(2));

      match req.send().await {
        Ok(resp) => {
          if resp.status().is_success() {
            return Ok(());
          }
          LOGGER.log_error("Bettercap", &format!("Request failed with status {}", resp.status()));
        }
        Err(e) => {
          if !e.is_connect() {
            // The server clearly got the request but didnt like it
            // Dont try this again.
            return Ok(());
          }
          LOGGER.log_error("Bettercap", &format!("Request failed: {e}"));
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
