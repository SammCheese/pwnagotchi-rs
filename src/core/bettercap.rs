use crate::core::config::Config;
use crate::core::log::LOGGER;

use std::{collections::HashMap, time::Duration};
use serde_json::Value;
use tokio::{net::TcpStream, time::sleep};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::protocol::Message;
use futures_util::stream::{SplitStream, StreamExt};

#[derive(serde::Deserialize)]
pub struct BettercapSession {
    pub interfaces: Vec<BInterfaces>,
    pub modules: HashMap<String, Value>,
}

#[derive(serde::Deserialize)]
pub struct BInterfaces {
    pub index: u32,
    pub mtu: u32,
    pub name: String,
    pub mac: String,
    pub vendor: String,
    pub flags: Vec<String>,
    pub addresses: Vec<BInterfaceAddress>,
}

#[derive(serde::Deserialize)]
pub struct BInterfaceAddress {
    pub address: String,
    pub r#type: String,
}


#[derive(Debug, Clone)]
pub struct Bettercap {
    pub bettercap_path: String,
    pub retries: u32,
    pub ping_timeout: u64,
    pub ping_interval: u64,
    pub max_queue: usize,
    pub min_sleep: f64,
    pub max_sleep: f64,
    pub is_ready: bool,

    hostname: String,
    port: u16,
    username: String,
    password: String,
    url: String,
    websocket_url: String,
    scheme: String,
}

impl Default for Bettercap {
    fn default() -> Self {
        Bettercap {
            bettercap_path: "/usr/bin/bettercap".into(),
            retries: 5,
            ping_timeout: 180,
            ping_interval: 15,
            max_queue: 10000,
            min_sleep: 0.5,
            max_sleep: 5.0,
            hostname: "localhost".into(),
            scheme: "http".into(),
            port: 8081,
            username: "user".into(),
            password: "pass".into(),
            url: "%{scheme}://%{username}:%{password}@%{hostname}:%{port}/api".into(),
            websocket_url: "ws://%{username}:%{password}@%{hostname}:%{port}/api".into(),
            is_ready: false,
        }
    }
}

type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

impl Bettercap {
    pub fn new(config: &Config) -> Self {
        let scheme = if config.bettercap.port == 443 { "https" } else { "http" };
        let ws_scheme = if config.bettercap.port == 443 { "wss" } else { "ws" };
        
        Bettercap {
            bettercap_path: config.main.bettercap_path.clone(),
            retries: 5,
            ping_timeout: 180,
            ping_interval: 15,
            max_queue: 10000,
            min_sleep: 0.5,
            max_sleep: 5.0,
            hostname: config.bettercap.hostname.clone(),
            port: config.bettercap.port,
            username: config.bettercap.username.clone(),
            password: config.bettercap.password.clone(),
            url: format!(
                "{}://{}:{}@{}:{}/api",
                scheme,
                config.bettercap.username,
                config.bettercap.password,
                config.bettercap.hostname,
                config.bettercap.port
            ),
            websocket_url: format!(
                "{}://{}:{}@{}:{}/api/events",
                ws_scheme,
                config.bettercap.username,
                config.bettercap.password,
                config.bettercap.hostname,
                config.bettercap.port
            ),
            scheme: scheme.to_string(),
            is_ready: false,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.is_ready
    }

    pub async fn session(&self, sess: Option<&str>) -> Result<BettercapSession, anyhow::Error> {
        let sess = sess.unwrap_or("session");
        let client = reqwest::Client::new();
        let url = format!("{}/{}", self.url, sess);
        let body = client.get(url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?
            .json::<BettercapSession>()
            .await?;

        Ok(body)
    }

    pub async fn start_websocket<F>(&mut self, mut consumer: F)
    where
        F: FnMut(String) -> Result<(), ()> + Send + 'static,
    {
        let ws_url = &self.websocket_url;

        loop {
            LOGGER.log_info("Bettercap", &format!("Connecting to WebSocket: {}", ws_url));
            match connect_async(ws_url).await {
                Ok((ws_stream, _)) => {
                    self.is_ready = true;
                    LOGGER.log_info("Bettercap", "WebSocket connected");
                    let (_, mut read) = ws_stream.split();

                    // Listen for messages
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(txt)) => {
                                let _ = consumer(txt.to_string());
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&txt) {
                                    if let Some(event_type) = json.get("type") {
                                        LOGGER.log_info("Bettercap", &format!("Received event type: {}", event_type));
                                    }
                                }
                            }
                            Ok(Message::Binary(_)) => {
                                LOGGER.log_info("Bettercap", "Received binary message");
                            }
                            Ok(Message::Close(_)) => {
                                LOGGER.log_info("Bettercap", "WebSocket closed by server.");
                                self.is_ready = false;
                                break;
                            }
                            Err(e) => {
                                LOGGER.log_error("Bettercap", &format!("WebSocket error: {e}"));
                                self.is_ready = false;
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    LOGGER.log_error("Bettercap", &format!("Failed to connect: {e}"));
                    self.is_ready = false;
                }
            }
            let sleep_time = self.min_sleep + (self.max_sleep - self.min_sleep) * rand::random::<f64>();
            LOGGER.log_info("Bettercap", &format!("Retrying in {:.1} seconds...", sleep_time));
            sleep(Duration::from_secs_f64(sleep_time)).await;
        }
    }

    pub async fn run(&self, args: &[&str]) -> Result<(), anyhow::Error> {
      let url = self.url
        .replace("%{scheme}", &self.scheme)
        .replace("%{username}", &self.username)
        .replace("%{password}", &self.password)
        .replace("%{hostname}", &self.hostname)
        .replace("%{port}", &self.port.to_string())
        + "/session";

      LOGGER.log_info("Bettercap", &format!("Commanding Bettercap to {}", args.join(" ")));

      let mut retries_left = self.retries;
      let client = reqwest::Client::new();

      loop {
        let req = client.post(&url)
          .json(&serde_json::json!({"cmd": args.join(" ")}))
          .header("Content-Type", "application/json")
          .basic_auth(&self.username, Some(&self.password));

        let res = req.send().await?;

        if res.status().is_success() {
          let body = res.text().await?;
          LOGGER.log_info("Bettercap", &format!("Response: {}", body));
          break Ok(());
        } else {
          let status = res.status();
          let body = res.text().await?;
          LOGGER.log_error("Bettercap", &format!("Error: {} - {}", status, body));
          if retries_left == 0 {
            break Err(anyhow::anyhow!("Max retries reached"));
          }
          LOGGER.log_info("Bettercap", &format!("Retrying in {} seconds...", self.ping_interval));
          retries_left -= 1;
          sleep(Duration::from_secs(self.ping_interval)).await;
        }
      }
    }
}