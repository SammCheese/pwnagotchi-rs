use crate::models::net::AccessPoint;

#[async_trait::async_trait]
pub trait AgentObserver: Send + Sync {
  fn on_miss(&self, who: &AccessPoint);
  fn on_error(&self, ap: &AccessPoint, err: &str);
  fn set_starting(&self);
  fn set_ready(&self);
  fn set_rebooting(&self);
  fn in_good_mood(&self) -> bool;
  fn set_grateful(&self);
  fn set_lonely(&self);
  fn set_bored(&self);
  fn set_sad(&self);
  fn set_angry(&self, factor: f32);
  fn set_excited(&self);
  async fn wait_for(&self, duration: u32, sleeping: Option<bool>);
  fn is_stale(&self) -> bool;
  fn any_activity(&self) -> bool;
  fn next_epoch(&self);
  fn has_support_network_for(&self, factor: f32) -> bool;
}
