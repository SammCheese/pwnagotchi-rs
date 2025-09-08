const API_ADDRESS: &str = "http://127.0.0.1:8666/api/v1";

use std::sync::LazyLock;

use pwnagotchi_shared::{
  log::LOGGER, models::grid::PeerResponse, sessions::lastsession::LastSession,
};
use ureq::{Agent, config::Config};

static CONFIG: LazyLock<Config> = LazyLock::new(|| {
  Agent::config_builder()
    .timeout_global(Some(std::time::Duration::from_secs(10)))
    .build()
});

static CLIENT: LazyLock<Agent> = LazyLock::new(|| Agent::new_with_config(CONFIG.clone()));

pub fn is_connected() -> bool {
  let host = "https://api.opwngrid.xyz/api/v1/uptime";
  let req = CLIENT
    .get(host)
    .header("User-Agent", format!("pwnagotchi-rs/{}", env!("CARGO_PKG_VERSION")))
    .call();

  req.ok().is_some_and(|r| r.status() == 200)
}

pub fn call<T>(endpoint: &str, data: &serde_json::Value) -> Option<T>
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
        match response.body_mut().read_json() {
          Ok(json) => Some(json),
          Err(e) => {
            LOGGER.log_error("GRID", &format!("Failed to parse JSON response: {e}"));
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

pub async fn advertise(enabled: Option<bool>) -> Option<serde_json::Value> {
  let enabled = enabled.unwrap_or(true);
  call(&format!("mesh/{enabled}"), &serde_json::Value::Null)
}

pub async fn set_advertisement_data(data: serde_json::Value) -> Option<serde_json::Value> {
  call("mesh/data", &data)
}

pub fn get_advertisement_data() -> Option<serde_json::Value> {
  call("mesh/data", &serde_json::Value::Null)
}

pub fn memory() -> Option<serde_json::Value> {
  call("system/memory", &serde_json::Value::Null)
}

pub async fn peers() -> Option<Vec<PeerResponse>> {
  call("mesh/peers", &serde_json::Value::Null)
}

/// Returns the closest peer from the list of peers, if any.
///
/// # Panics
///
/// This function will panic if the value returned by `peers()` is not an array.
pub async fn closest_peer() -> Option<PeerResponse> {
  let all = peers().await?;
  if all.is_empty() { None } else { Some(all.first().cloned().unwrap()) }
}

pub fn update_data(last_session: &LastSession) {
  let data = serde_json::json!({
    "ai": "No AI!",
    "session": {
      "duration": last_session.duration,
      "epochs": last_session.epochs,
      "train_epochs": last_session.train_epochs,
      "avg_reward": last_session.avg_reward,
      "min_reward": last_session.min_reward,
      "max_reward": last_session.max_reward,
      "deauthed": last_session.deauthed,
      "associated": last_session.associated,
      "handshakes": last_session.handshakes,
      "peers": last_session.peers,
    },
    "uname": "linux",
    "version": env!("CARGO_PKG_VERSION"),
    "build": "Pwnagotchi-rs by Sammy!",
    "plugins": [],
    "language": "en",
    "bettercap": "1.0.0",
    "opwngrid": "1.1.0",
  });

  LOGGER.log_debug("GRID", "Updating Grid Data!");
  call::<()>("data", &data);
}

pub fn report_ap(essid: &str, bssid: &str) {
  let data = serde_json::json!({
    "essid": essid,
    "bssid": bssid,
  });

  LOGGER.log_debug("GRID", &format!("Reporting AP {essid} ({bssid})"));
  call::<()>("report/ap", &data);
}

pub fn inbox(page: Option<u32>, with_pager: Option<bool>) -> Option<serde_json::Value> {
  let page = page.unwrap_or(1);
  let with_pager = with_pager.unwrap_or(false);
  let res = call(&format!("inbox?p={page}"), &serde_json::Value::Null);
  if with_pager { res } else { res.and_then(|r| r.get("messages").cloned()) }
}

pub fn inbox_message(id: &str) -> Option<serde_json::Value> {
  call(&format!("inbox/{id}"), &serde_json::Value::Null)
}

pub fn mark_message(id: &str, mark: &str) -> Option<serde_json::Value> {
  call(&format!("inbox/{id}/mark"), &serde_json::json!({ "read": mark }))
}

/*pub async fn send_message(to: &str, message: &str) -> Option<serde_json::Value> {
  call(&format!("unit/{to}/send")).await
}
*/
