use std::collections::HashMap;
use crate::core::mesh::wifi;

#[derive(Clone)]
pub struct RewardFunction;

impl RewardFunction {
  const RANGE: (f64, f64) = (-0.7, 1.02);
  const NOZERO: f64 = 1e-20;

  pub fn call(&self, epoch: u64, state: &HashMap<&str, f64>) -> f64 {
        let tot_epochs = epoch as f64 + Self::NOZERO;
        let tot_interactions =
            (state.get("num_deauths").unwrap_or(&0.0)
            + state.get("num_associations").unwrap_or(&0.0))
            .max(*state.get("num_handshakes").unwrap_or(&0.0)) + Self::NOZERO;

        let tot_channels = wifi::NUM_CHANNELS as f64;

        let h = state.get("num_handshakes").unwrap_or(&0.0) / tot_interactions;
        let a = 0.2 * (state.get("active_for_epochs").unwrap_or(&0.0) / tot_epochs);
        let c = 0.1 * (state.get("num_hops").unwrap_or(&0.0) / tot_channels);

        let b = -0.3 * (state.get("blind_for_epochs").unwrap_or(&0.0) / tot_epochs);
        let m = -0.3 * (state.get("missed_interactions").unwrap_or(&0.0) / tot_interactions);
        let i = -0.2 * (state.get("inactive_for_epochs").unwrap_or(&0.0) / tot_epochs);

        let sad = *state.get("sad_for_epochs").unwrap_or(&0.0);
        let bored = *state.get("bored_for_epochs").unwrap_or(&0.0);

        let s = if sad >= 5.0 { -0.2 * (sad / tot_epochs) } else { 0.0 };
        let l = if bored >= 5.0 { -0.1 * (bored / tot_epochs) } else { 0.0 };

        h + a + c + b + m + i + s + l
  }
}