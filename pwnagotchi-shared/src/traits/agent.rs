use crate::{
  models::{
    agent::RunningMode,
    net::{AccessPoint, Station},
  },
  traits::general::CoreModule,
};

#[async_trait::async_trait]
pub trait AgentTrait: Send + Sync + CoreModule {
  async fn set_mode(&self, mode: RunningMode);
  async fn recon(&self);
  async fn associate(&self, ap: &AccessPoint, throttle: Option<f32>);
  async fn deauth(&self, ap: &AccessPoint, sta: &Station, throttle: Option<f32>);
  async fn set_channel(&self, channel: u8);
  async fn get_access_points_by_channel(&self) -> Vec<(u8, Vec<AccessPoint>)>;
  fn start_pwnagotchi(&self);
  fn reboot(&self);
  fn restart(&self, mode: Option<RunningMode>);
}
