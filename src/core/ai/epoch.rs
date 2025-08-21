use std::{ collections::HashMap, sync::mpsc::{ sync_channel, Receiver, SyncSender }, time::{ Duration, Instant } };
use crate::core::{ ai::reward::RewardFunction, config::Config };

pub struct Epoch {
    obs_tx: SyncSender<Observation>,
    obs_rx: Receiver<Observation>,
    data_tx: SyncSender<EpochData>,
    data_rx: Receiver<EpochData>,
    pub epoch: u64,
    pub config: Config,

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
    pub epoch_data: EpochData,
    reward: RewardFunction,
}

#[derive(Clone)]
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
struct Observation {
    aps_histogram: Vec<f32>,
    sta_histogram: Vec<f32>,
    peers_histogram: Vec<f32>,
}

impl Default for Epoch {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Observation {
    fn default() -> Self {
        Self {
            aps_histogram: vec![0.0; 256],
            sta_histogram: vec![0.0; 256],
            peers_histogram: vec![0.0; 256],
        }
    }
}

impl Default for EpochData {
    fn default() -> Self {
        Self {
            duration_secs: 0.0,
            slept_for_secs: 0.0,
            blind_for_epochs: 0,
            inactive_for_epochs: 0,
            active_for_epochs: 0,
            sad_for_epochs: 0,
            bored_for_epochs: 0,
            missed_interactions: 0,
            num_hops: 0,
            num_peers: 0,
            tot_bond: 0.0,
            avg_bond: 0.0,
            num_deauths: 0,
            num_associations: 0,
            num_handshakes: 0,
            cpu_load: 0.0,
            mem_usage: 0.0,
            temperature: 0.0,
            reward: 0.0,
        }
    }
}

impl Epoch {
    pub fn new() -> Self {
        let (obs_tx, obs_rx) = sync_channel(1);
        let (data_tx, data_rx) = sync_channel(1);

        Epoch {
            obs_tx,
            obs_rx,
            data_tx,
            data_rx,
            epoch: 0,
            config: Config::default(),
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
            epoch_data: EpochData::default(),
            reward: RewardFunction,
        }
    }

    pub fn observe(
        &mut self,
        aps_per_chan: Vec<f32>,
        sta_per_chan: Vec<f32>,
        peers_per_chan: Vec<f32>
    ) {
        self.observation = Observation {
            aps_histogram: aps_per_chan,
            sta_histogram: sta_per_chan,
            peers_histogram: peers_per_chan,
        };
        let _ = self.obs_tx.try_send(self.observation.clone());
    }

    pub fn next(&mut self) {
        self.epoch_duration = self.epoch_start.elapsed().as_secs_f64();

        self.epoch_data = EpochData {
            duration_secs: self.epoch_duration,
            slept_for_secs: self.num_slept as f64,
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
            reward: self.reward.call(self.epoch as f64, &self.reward_state()),
            cpu_load: 0.0,
            mem_usage: 0.0,
            temperature: 0.0,
        };

        self.data_tx.try_send(self.epoch_data.clone()).ok();

        self.epoch += 1;
        self.epoch_start = Instant::now();
        self.did_deauth = false;
        self.num_deauths = 0;
        self.did_associate = false;
        self.num_assocs = 0;
        self.num_missed = 0;
        self.did_handshakes = false;
        self.num_handshakes = 0;
        self.num_hops = 0;
        self.num_slept = 0;
        self.any_activity = false;
        self.num_peers = 0;
        self.total_bond_factor = 0.0;
        self.avg_bond_factor = 0.0;
    }

    pub fn wait_for_epoch_data(
        &self,
        with_observation: bool,
        timeout: Option<Duration>
    ) -> Option<(Option<Observation>, EpochData)> {
        let data = match timeout {
            Some(t) => self.data_rx.recv_timeout(t).ok()?,
            None => self.data_rx.recv().ok()?,
        };

        if with_observation {
            let obs = self.obs_rx.try_recv().ok();
            Some((obs, data))
        } else {
            Some((None, data))
        }
    }

    fn data(self) -> EpochData {
        return self.epoch_data;
    }

    fn reward_state(&self) -> HashMap<&str, f64> {
        let mut state = std::collections::HashMap::new();
        state.insert("num_deauths", self.num_deauths as f64);
        state.insert("num_associations", self.num_assocs as f64);
        state.insert("num_handshakes", self.num_handshakes as f64);
        state.insert("active_for_epochs", self.active_for as f64);
        state.insert("blind_for_epochs", self.blind_for as f64);
        state.insert("inactive_for_epochs", self.inactive_for as f64);
        state.insert("missed_interactions", self.num_missed as f64);
        state.insert("num_hops", self.num_hops as f64);
        state.insert("sad_for_epochs", self.sad_for as f64);
        state.insert("bored_for_epochs", self.bored_for as f64);
        state
    }
}
