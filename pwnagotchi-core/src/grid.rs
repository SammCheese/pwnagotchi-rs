const API_ADDRESS: &str = "http://127.0.0.1:8666/api/v1";

use std::{process::Command, sync::LazyLock};

use pwnagotchi_shared::{
  config::config_read,
  logger::LOGGER,
  models::grid::PeerResponse,
  sessions::session_stats::SessionStats,
  traits::{general::CoreModule, grid::GridTrait},
  types::grid::GridDataResponse,
};
use ureq::{Agent, config::Config};

static GRID_CONFIG: LazyLock<Config> = LazyLock::new(|| {
  Agent::config_builder()
    .timeout_global(Some(std::time::Duration::from_secs(10)))
    .build()
});

static CLIENT: LazyLock<Agent> = LazyLock::new(|| Agent::new_with_config(GRID_CONFIG.clone()));

pub struct Grid;

impl CoreModule for Grid {
  fn name(&self) -> &'static str {
    "Grid"
  }
}

impl Default for Grid {
  fn default() -> Self {
    Self::new()
  }
}

impl Grid {
  pub fn new() -> Self {
    Self
  }

  fn call<T>(&self, endpoint: &str, data: &serde_json::Value) -> Option<T>
  where
    T: serde::de::DeserializeOwned,
  {
    let url = format!("{API_ADDRESS}/{endpoint}");

    let req = if data.is_null() {
      CLIENT
        .get(&url)
        .header("User-Agent", format!("pwnagotchi-rs/{}", env!("CARGO_PKG_VERSION")))
        .call()
    } else {
      CLIENT
        .post(&url)
        .header("User-Agent", format!("pwnagotchi-rs/{}", env!("CARGO_PKG_VERSION")))
        .send_json(data)
    };

    match req {
      Ok(mut response) => {
        if response.status() == 200 {
          match response.body_mut().read_json::<T>() {
            Ok(json) => Some(json),
            Err(e) => {
              LOGGER.log_error("GRID", &format!("Failed to parse JSON response: {e}"));
              LOGGER.log_debug(
                "GRID",
                &format!(
                  "Response body: {}",
                  response.body_mut().read_to_string().unwrap_or_default()
                ),
              );
              None
            }
          }
        } else {
          LOGGER.log_debug("GRID", &format!("Request failed with status: {}", response.status()));
          None
        }
      }
      Err(e) => {
        LOGGER.log_debug("GRID", &format!("HTTP request error: {e}"));
        None
      }
    }
  }
}

#[async_trait::async_trait]
impl GridTrait for Grid {
  fn is_connected(&self) -> bool {
    let host = "https://api.opwngrid.xyz/api/v1/uptime";
    let req = CLIENT
      .get(host)
      .header("User-Agent", format!("pwnagotchi-rs/{}", env!("CARGO_PKG_VERSION")))
      .call();

    match req {
      Ok(mut response) => {
        let json = response.body_mut().read_json::<serde_json::Value>();
        match json {
          Ok(value) => value.get("isUp").and_then(|v| v.as_bool()).unwrap_or(false),
          Err(_) => false,
        }
      }
      Err(_) => false,
    }
  }

  fn advertise(&self, enabled: Option<bool>) -> Option<serde_json::Value> {
    let enabled = enabled.unwrap_or(true);
    self.call(&format!("mesh/{enabled}"), &serde_json::Value::Null)
  }

  fn set_advertisement_data(&self, data: serde_json::Value) -> Option<serde_json::Value> {
    self.call("mesh/data", &data)
  }

  fn get_advertisement_data(&self) -> Option<serde_json::Value> {
    self.call("mesh/data", &serde_json::Value::Null)
  }

  fn memory(&self) -> Option<serde_json::Value> {
    self.call("system/memory", &serde_json::Value::Null)
  }

  async fn peers(&self) -> Option<Vec<PeerResponse>> {
    self.call("mesh/peers", &serde_json::Value::Null)
  }

  /// Returns the closest peer from the list of peers, if any.
  ///
  /// # Panics
  ///
  /// This function will panic if the value returned by `peers()` is not an
  /// array.
  async fn closest_peer(&self) -> Option<PeerResponse> {
    let all = self.peers().await?;
    if all.is_empty() { None } else { Some(all.first().cloned().unwrap()) }
  }

  fn update_data(&self, last_session: &SessionStats) {
    let uname = Command::new("uname")
      .arg("-a")
      .output()
      .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
      .unwrap_or_else(|_| "unknown".to_string());

    let num_epochs = last_session.epochs.epochs;
    let num_peers = last_session.peers.peers;

    let avg_reward_int = (last_session.epochs.avg_reward * 1000.0).round() as i32;
    let max_reward_int = (last_session.epochs.max_reward * 1000.0).round() as i32;
    let min_reward_int = (last_session.epochs.min_reward * 1000.0).round() as i32;

    let data = serde_json::json!({
      "ai": "No AI!",
      "session": {
        "duration": last_session.duration_human().unwrap_or("00:00:00".to_string()),
        "epochs": num_epochs,
        "train_epochs": last_session.epochs.train_epochs,
        "avg_reward": avg_reward_int,
        "min_reward": min_reward_int,
        "max_reward": max_reward_int,
        "deauthed": last_session.deauthed,
        "associated": last_session.associated,
        "handshakes": last_session.handshakes,
        "peers": num_peers,
      },
      "uname": uname,
      "version": env!("CARGO_PKG_VERSION"),
      "build": "Pwnagotchi-rs by Sammy!",
      "plugins": [],
      "language": config_read().main.lang.to_string(),
      "bettercap": "2.41.4",
      "opwngrid": "1.11.4",
    });

    LOGGER.log_debug(
      "GRID",
      &format!("Updating Grid Data: {}", serde_json::to_string(&data).unwrap_or_default()),
    );

    self.call::<GridDataResponse>("data", &data);
  }

  fn report_ap(&self, essid: &str, bssid: &str) {
    let data = serde_json::json!({
      "essid": essid,
      "bssid": bssid,
    });

    LOGGER.log_debug("GRID", &format!("Reporting AP {essid} ({bssid})"));
    self.call::<()>("report/ap", &data);
  }

  fn inbox(&self, page: Option<u32>, with_pager: Option<bool>) -> Option<serde_json::Value> {
    let page = page.unwrap_or(1);
    let with_pager = with_pager.unwrap_or(false);
    let res = self.call(&format!("inbox?p={page}"), &serde_json::Value::Null);
    if with_pager { res } else { res.and_then(|r| r.get("messages").cloned()) }
  }

  fn inbox_message(&self, id: &str) -> Option<serde_json::Value> {
    self.call(&format!("inbox/{id}"), &serde_json::Value::Null)
  }

  fn mark_message(&self, id: &str, mark: &str) -> Option<serde_json::Value> {
    self.call(&format!("inbox/{id}/mark"), &serde_json::json!({ "read": mark }))
  }

  /*pub async fn send_message(to: &str, message: &str) -> Option<serde_json::Value> {
    Self::call(&format!("unit/{to}/send")).await
  }
  */
}
