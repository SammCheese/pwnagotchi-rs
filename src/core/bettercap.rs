use crate::core::config::Config;
use crate::core::log::LOGGER;

use std::time::Duration;
use tokio::{net::TcpStream, time::sleep};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::protocol::Message;
use futures_util::stream::{SplitStream, StreamExt};

#[derive(Debug, Clone)]
pub struct Bettercap {
    pub bettercap_path: String,
    pub retries: u32,
    pub ping_timeout: u64,
    pub ping_interval: u64,
    pub max_queue: usize,
    pub min_sleep: f64,
    pub max_sleep: f64,

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
        }
    }
}

type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

impl Bettercap {
    pub fn new(config: &Config) -> Self {
        let bettercap_path = config.get_config::<String>("main.bettercap_path", "/usr/bin/bettercap".to_string());
        let retries = config.get_config::<u32>("bettercap.retries", 3);
        let ping_timeout = config.get_config::<u64>("bettercap.ping_timeout", 1000);
        let ping_interval = config.get_config::<u64>("bettercap.ping_interval", 10000);
        let max_queue = config.get_config::<usize>("bettercap.max_queue", 100);
        let min_sleep = config.get_config::<f64>("bettercap.min_sleep", 100.0);
        let max_sleep = config.get_config::<f64>("bettercap.max_sleep", 1000.0);
        let hostname = config.get_config::<String>("bettercap.hostname", "localhost".to_string());
        let port = config.get_config::<u16>("bettercap.port", 8081);
        let username = config.get_config::<String>("bettercap.username", "user".to_string());
        let password = config.get_config::<String>("bettercap.password", "pass".to_string());
        let scheme = if port == 443 { "https".to_string() } else { "http".to_string() };

        Bettercap {
            bettercap_path,
            retries,
            ping_timeout,
            ping_interval,
            max_queue,
            min_sleep,
            max_sleep,
            hostname: hostname.clone(),
            port,
            username: username.clone(),
            password: password.clone(),
            url: format!("{}://{}:{}@{}:{}/api", scheme, username, password, hostname, port),
            websocket_url: format!("ws://{}:{}@{}:{}/api", username, password, hostname, port),
            scheme,
            ..Default::default()
        }
    }

    pub async fn start_websocket<F>(&self, mut consumer: F)
    where
        F: FnMut(String) -> Result<(), ()> + Send + 'static,
    {
        let ws_url = self.websocket_url
            .replace("%{scheme}", &self.scheme)
            .replace("%{username}", &self.username)
            .replace("%{password}", &self.password)
            .replace("%{hostname}", &self.hostname)
            .replace("%{port}", &self.port.to_string())
            + "/events";

        loop {
            LOGGER.log_info("Bettercap", &format!("Connecting to WebSocket: {}", ws_url));
            match connect_async(&ws_url).await {
                Ok((ws_stream, _)) => {
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
                                break;
                            }
                            Err(e) => {
                                LOGGER.log_error("Bettercap", &format!("WebSocket error: {e}"));
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    LOGGER.log_error("Bettercap", &format!("Failed to connect: {e}"));
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