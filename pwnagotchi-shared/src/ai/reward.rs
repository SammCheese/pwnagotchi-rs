use crate::{traits::epoch::EpochData, utils::wifi};

const NOZERO: f64 = 1e-20;

pub fn calculate_reward(epoch: u64, state: &EpochData) -> f64 {
  #[allow(clippy::cast_precision_loss)]
  let tot_epochs = epoch as f64 + NOZERO;

  let tot_interactions = (f64::from(state.num_deauths) + f64::from(state.num_associations))
    .max(f64::from(state.num_handshakes))
    + NOZERO;
  let tot_channels = f64::from(wifi::NUM_CHANNELS);

  let hs: f64 = f64::from(state.num_handshakes) / tot_interactions;
  let ac = 0.2 * (f64::from(state.active_for_epochs) / tot_epochs);
  let chps = 0.1 * (f64::from(state.num_hops) / tot_channels);

  let blind = -0.3 * (f64::from(state.blind_for_epochs) / tot_epochs);
  let missed = -0.3 * (f64::from(state.missed_interactions) / tot_interactions);
  let inactive = -0.2 * (f64::from(state.inactive_for_epochs) / tot_epochs);

  let sad = if state.sad_for_epochs >= 5 { f64::from(state.sad_for_epochs) } else { 0.0 };
  let bored = if state.bored_for_epochs >= 5 { f64::from(state.bored_for_epochs) } else { 0.0 };

  let sad_tot = -0.2 * (sad / tot_epochs);
  let bored_tot = -0.1 * (bored / tot_epochs);

  hs + ac + chps + blind + missed + inactive + sad_tot + bored_tot
}
