#![allow(
  clippy::cast_possible_truncation,
  clippy::cast_precision_loss,
  clippy::struct_excessive_bools
)]

use std::{
  mem::take,
  time::{Duration, Instant},
  vec,
};

use pwnagotchi_shared::{
  config::config, logger::LOGGER, mesh::peer::Peer, models::net::AccessPoint,
  types::epoch::Activity,
};
use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::{ai::reward::RewardFunction, mesh::wifi};

pub struct Epoch {
  obs_tx: Sender<Observation>,
  obs_rx: Receiver<Observation>,
  data_tx: Sender<EpochData>,
  data_rx: Receiver<EpochData>,
  pub epoch: u64,

  pub inactive_for: u32,
  pub active_for: u32,
  pub blind_for: u32,
  pub sad_for: u32,
  pub bored_for: u32,

  pub did_deauth: bool,
  pub num_deauths: u32,
  pub did_associate: bool,
  pub num_assocs: u32,
  pub num_missed: u32,
  pub did_handshakes: bool,
  pub num_handshakes: u32,
  pub num_hops: u32,
  pub num_slept: u32,
  pub num_peers: u32,
  pub total_bond_factor: f32,
  pub avg_bond_factor: f32,
  pub any_activity: bool,

  pub epoch_start: Instant,
  pub epoch_duration: f64,

  pub non_overlapping_channels: Vec<String>,
  pub observation: Observation,
  pub observation_ready: bool,
  pub epoch_data: EpochData,
  pub epoch_data_ready: bool,
}

#[derive(Clone, Default)]
pub struct EpochData {
  pub duration_secs: f64,
  pub slept_for_secs: f64,
  pub blind_for_epochs: u32,
  pub inactive_for_epochs: u32,
  pub active_for_epochs: u32,
  pub sad_for_epochs: u32,
  pub bored_for_epochs: u32,
  pub missed_interactions: u32,
  pub num_hops: u32,
  pub num_peers: u32,
  pub tot_bond: f32,
  pub avg_bond: f32,
  pub num_deauths: u32,
  pub num_associations: u32,
  pub num_handshakes: u32,
  pub cpu_load: f32,
  pub mem_usage: f32,
  pub temperature: f32,
  pub reward: f64,
}

#[derive(Clone)]
pub struct Observation {
  pub aps: Vec<f32>,
  pub sta: Vec<f32>,
  pub peers: Vec<f32>,
}

impl Default for Epoch {
  fn default() -> Self {
    Self::new()
  }
}

impl Default for Observation {
  fn default() -> Self {
    Self {
      aps: vec![0.0; 256],
      sta: vec![0.0; 256],
      peers: vec![0.0; 256],
    }
  }
}

impl Epoch {
  pub fn new() -> Self {
    let (obs_tx, obs_rx) = channel(1);
    let (data_tx, data_rx) = channel(1);

    Self {
      obs_tx,
      obs_rx,
      data_tx,
      data_rx,
      epoch: 0,
      inactive_for: 0,
      active_for: 0,
      blind_for: 0,
      sad_for: 0,
      bored_for: 0,
      did_deauth: false,
      num_deauths: 0,
      did_associate: false,
      num_assocs: 0,
      num_missed: 0,
      did_handshakes: false,
      num_handshakes: 0,
      num_hops: 0,
      num_slept: 0,
      num_peers: 0,
      total_bond_factor: 0.0,
      avg_bond_factor: 0.0,
      any_activity: false,
      epoch_start: Instant::now(),
      epoch_duration: 0.0,
      non_overlapping_channels: Vec::new(),
      observation: Observation::default(),
      observation_ready: false,
      epoch_data: EpochData::default(),
      epoch_data_ready: false,
    }
  }

  pub fn observe(&mut self, aps: &Vec<AccessPoint>, peers: &Vec<Peer>) {
    let num_aps = aps.len();

    if num_aps == 0 {
      self.blind_for += 1;
    } else {
      self.blind_for = 0;
    }

    let bond_unit_scale = config().personality.bond_encounters_factor;

    self.num_peers = peers.len().try_into().unwrap_or(0);
    self.total_bond_factor = aps
      .iter()
      .map(|ap| {
        #[allow(clippy::cast_possible_truncation)]
        let bond_factor = (f64::from(ap.rssi) / f64::from(bond_unit_scale)) as f32;

        if bond_factor < 0.0 { 0.0 } else { bond_factor }
      })
      .sum::<f32>();
    self.avg_bond_factor = if num_aps > 0 { self.total_bond_factor / num_aps as f32 } else { 0.0 };

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    let num_aps_f = (aps.len() as f64) as f32 + 1e-10;
    let num_sta = aps.iter().map(|ap| ap.clients.len() as u32 as f32).sum::<f32>() / num_aps_f;
    let num_channels = usize::try_from(wifi::NUM_CHANNELS).unwrap_or(0);
    let mut aps_per_chan = vec![0.0; num_channels];
    let mut sta_per_chan = vec![0.0; num_channels];
    let mut peers_per_chan = vec![0.0; num_channels];

    for ap in aps {
      let ch_idx = ap.channel - 1;
      aps_per_chan[ch_idx as usize] += 1.0;
      sta_per_chan[ch_idx as usize] += ap.clients.len() as f32;
    }

    for peer in peers {
      let ch_idx = peer.last_channel - 1;
      if ch_idx < num_channels as u8 {
        peers_per_chan[ch_idx as usize] += 1.0;
      }
    }

    #[allow(clippy::cast_possible_truncation)]
    let num_peers_f = (f64::from(self.num_peers) + 1e-10) as f32;

    aps_per_chan = aps_per_chan.iter().map(|x| *x / num_aps_f).collect();
    sta_per_chan = sta_per_chan.iter().map(|x| *x / num_sta).collect();
    peers_per_chan = peers_per_chan.iter().map(|x| *x / num_peers_f).collect();

    self.observation = Observation {
      aps: aps_per_chan,
      sta: sta_per_chan,
      peers: peers_per_chan,
    };

    let _ = self.obs_tx.try_send(take(&mut self.observation));
  }

  pub fn next(&mut self) {
    if !self.any_activity && !self.did_handshakes {
      self.inactive_for += 1;
      self.active_for = 0;
    } else {
      self.active_for += 1;
      self.inactive_for = 0;
      self.sad_for = 0;
      self.bored_for = 0;
    }

    if self.inactive_for >= config().personality.sad_num_epochs {
      self.bored_for = 0;
      self.sad_for += 1;
    } else if self.inactive_for >= config().personality.bored_num_epochs {
      self.sad_for = 0;
      self.bored_for += 1;
    } else {
      self.sad_for = 0;
      self.bored_for = 0;
    }

    let now = Instant::now();
    self.epoch_duration = now.duration_since(self.epoch_start).as_secs_f64();

    self.epoch_data = EpochData {
      duration_secs: self.epoch_duration,
      slept_for_secs: f64::from(self.num_slept),
      blind_for_epochs: self.blind_for,
      inactive_for_epochs: self.inactive_for,
      active_for_epochs: self.active_for,
      sad_for_epochs: self.sad_for,
      bored_for_epochs: self.bored_for,
      missed_interactions: self.num_missed,
      num_hops: self.num_hops,
      num_peers: self.num_peers,
      tot_bond: self.total_bond_factor,
      avg_bond: self.avg_bond_factor,
      num_deauths: self.num_deauths,
      num_associations: self.num_assocs,
      num_handshakes: self.num_handshakes,
      cpu_load: 0.0,
      mem_usage: 0.0,
      temperature: 0.0,
      reward: RewardFunction::call(self.epoch + 1, &self.epoch_data),
    };

    LOGGER.log_info(format!("Epoch {}", self.epoch).as_str(), format!(
      "duration={} slept_for={} blind={} sad={} bored={} inactive={} active={} peers={} tot_bond={} avg_bond={} hops={} missed={} deauths={} assocs={} handshakes={} cpu={} mem={} temperature={} reward={}",
      self.epoch_data.duration_secs,
      self.epoch_data.slept_for_secs,
      self.epoch_data.blind_for_epochs,
      self.epoch_data.sad_for_epochs,
      self.epoch_data.bored_for_epochs,
      self.epoch_data.inactive_for_epochs,
      self.epoch_data.active_for_epochs,
      self.epoch_data.num_peers,
      self.epoch_data.tot_bond,
      self.epoch_data.avg_bond,
      self.epoch_data.num_hops,
      self.epoch_data.missed_interactions,
      self.epoch_data.num_deauths,
      self.epoch_data.num_associations,
      self.epoch_data.num_handshakes,
      self.epoch_data.cpu_load,
      self.epoch_data.mem_usage,
      self.epoch_data.temperature,
      self.epoch_data.reward,
    ).as_str());

    self.epoch_data_ready = true;
    self.data_tx.try_send(take(&mut self.epoch_data)).ok();

    self.epoch += 1;
    self.epoch_start = Instant::now();
    self.did_deauth = false;
    self.num_deauths = 0;
    self.num_peers = 0;
    self.total_bond_factor = 0.0;
    self.avg_bond_factor = 0.0;
    self.did_associate = false;
    self.num_assocs = 0;
    self.num_missed = 0;
    self.did_handshakes = false;
    self.num_handshakes = 0;
    self.num_hops = 0;
    self.num_slept = 0;
    self.any_activity = false;
  }

  pub fn track(&mut self, activity: Activity, increment: Option<u32>) {
    match activity {
      Activity::Deauth => {
        self.did_deauth = true;
        self.num_deauths += increment.unwrap_or(1);
        self.any_activity = true;
      }
      Activity::Association => {
        self.did_associate = true;
        self.num_assocs += increment.unwrap_or(1);
        self.any_activity = true;
      }
      Activity::Miss => {
        self.num_missed += increment.unwrap_or(1);
      }
      Activity::Hop => {
        self.num_hops += increment.unwrap_or(1);
        self.did_deauth = false;
        self.did_associate = false;
      }
      Activity::Handshake => {
        self.num_handshakes += increment.unwrap_or(1);
        self.did_handshakes = true;
      }
      Activity::Sleep => {
        self.num_slept += increment.unwrap_or(1);
      }
    }
  }

  pub async fn wait_for_epoch_data(
    &mut self,
    with_observation: bool,
    timeout: Option<Duration>,
  ) -> Option<(Option<Observation>, EpochData)> {
    let data = match timeout {
      Some(t) => match tokio::time::timeout(t, self.data_rx.recv()).await {
        Ok(Some(data)) => data,
        _ => return None,
      },
      None => self.data_rx.recv().await?,
    };

    if with_observation {
      let obs = self.obs_rx.try_recv().ok();
      Some((obs, data))
    } else {
      Some((None, data))
    }
  }

  #[allow(dead_code)]
  fn data(self) -> EpochData {
    self.epoch_data
  }
}
