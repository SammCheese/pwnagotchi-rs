use crate::{models::grid::PeerResponse, sessions::session_stats::SessionStats};

#[async_trait::async_trait]
pub trait GridTrait: Send + Sync {
  fn is_connected(&self) -> bool;
  fn advertise(&self, enabled: Option<bool>) -> Option<serde_json::Value>;
  fn set_advertisement_data(&self, data: serde_json::Value) -> Option<serde_json::Value>;
  fn get_advertisement_data(&self) -> Option<serde_json::Value>;
  fn memory(&self) -> Option<serde_json::Value>;
  async fn peers(&self) -> Option<Vec<PeerResponse>>;
  async fn closest_peer(&self) -> Option<PeerResponse>;
  fn update_data(&self, last_session: &SessionStats);
  fn report_ap(&self, essid: &str, bssid: &str);
  fn inbox(&self, page: Option<u32>, with_pager: Option<bool>) -> Option<serde_json::Value>;
  fn inbox_message(&self, id: &str) -> Option<serde_json::Value>;
  fn mark_message(&self, id: &str, mark: &str) -> Option<serde_json::Value>;
}
