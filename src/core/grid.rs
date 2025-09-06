const API_ADDRESS: &str = "http://127.0.0.1:8666/api/v1/";

use std::sync::LazyLock;

use crate::core::{
  log::LOGGER, mesh::advertiser::Advertisement, sessions::lastsession::LastSession,
};

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub async fn is_connected() -> bool {
  let host = "https://api.opwngrid.xyz/api/v1/uptime";
  let req = CLIENT
    .get(host)
    .header("User-Agent", format!("pwnagotchi-rs/{}", env!("CARGO_PKG_VERSION")))
    .timeout(std::time::Duration::from_secs(30))
    .send();
  req.await.is_ok()
}

pub async fn call<T>(endpoint: &str, data: serde_json::Value) -> Option<T>
where
  T: serde::de::DeserializeOwned,
{
  let url = format!("{API_ADDRESS}/{endpoint}");

  let req = if data.is_null() {
    CLIENT
      .get(&url)
      .header("User-Agent", format!("pwnagotchi-rs/{}", env!("CARGO_PKG_VERSION")))
      .timeout(std::time::Duration::from_secs(10))
      .send()
  } else {
    CLIENT
      .post(&url)
      .header("User-Agent", format!("pwnagotchi-rs/{}", env!("CARGO_PKG_VERSION")))
      .json(&data)
      .timeout(std::time::Duration::from_secs(10))
      .send()
  };

  match req.await {
    Ok(resp) => resp.json::<T>().await.ok(),
    Err(_) => None,
  }
}

pub async fn advertise(enabled: Option<bool>) -> Option<serde_json::Value> {
  let enabled = enabled.unwrap_or(true);
  call(&format!("mesh/{enabled}"), serde_json::Value::Null).await
}

pub async fn set_advertisement_data(data: serde_json::Value) -> Option<serde_json::Value> {
  call("mesh/data", data).await
}

pub async fn get_advertisement_data() -> Option<serde_json::Value> {
  call("mesh/data", serde_json::Value::Null).await
}

pub async fn memory() -> Option<serde_json::Value> {
  call("system/memory", serde_json::Value::Null).await
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct PeerResponse {
  pub fingerprint: Option<String>,
  pub met_at: Option<String>,
  pub encounters: Option<u32>,
  pub prev_seen_at: Option<String>,
  pub detected_at: Option<String>,
  pub seen_at: Option<String>,
  pub channel: Option<u8>,
  pub rssi: Option<i16>,
  pub session_id: Option<String>,
  pub advertisement: Option<Advertisement>,
}

pub async fn peers() -> Option<Vec<PeerResponse>> {
  call("mesh/peers", serde_json::Value::Null).await
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

pub async fn update_data(last_session: LastSession) {
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
  call::<()>("data", data).await;
}

pub async fn report_ap(essid: &str, bssid: &str) {
  let data = serde_json::json!({
    "essid": essid,
    "bssid": bssid,
  });

  LOGGER.log_debug("GRID", &format!("Reporting AP {essid} ({bssid})"));
  call::<()>("report/ap", data).await;
}

pub async fn inbox(page: Option<u32>, with_pager: Option<bool>) -> Option<serde_json::Value> {
  let page = page.unwrap_or(1);
  let with_pager = with_pager.unwrap_or(false);
  let res = call(&format!("inbox?p={page}"), serde_json::Value::Null).await;
  if with_pager { res } else { res.and_then(|r| r.get("messages").cloned()) }
}

pub async fn inbox_message(id: &str) -> Option<serde_json::Value> {
  call(&format!("inbox/{id}"), serde_json::Value::Null).await
}

pub async fn mark_message(id: &str, mark: String) -> Option<serde_json::Value> {
  call(&format!("inbox/{id}/mark"), serde_json::json!({ "read": mark })).await
}

/*pub async fn send_message(to: &str, message: &str) -> Option<serde_json::Value> {
  call(&format!("unit/{to}/send")).await
}
*/
