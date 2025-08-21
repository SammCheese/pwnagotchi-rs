use crate::core::{ai::Epoch, config::Config, log::LOGGER};

pub struct Automata {
  config: Config,
  pub epoch: Epoch
}

impl Default for Automata {
  fn default() -> Self {
    let config = Config::default();
    let epoch = Epoch::new();
    Self { config, epoch }
  }
}

impl Automata {
  pub fn new(config: Config) -> Self {
    let epoch = Epoch::new();
    Self { config, epoch }
  }

  fn on_miss(&mut self) {
    LOGGER.log_info("Automata", "Missed an interaction :(");
    self.epoch.track("miss", None);
  }

  fn on_error(&mut self, error: &str) {
    LOGGER.log_error("Automata", error);
    if error.contains("is an unknown BSSID") {
      self.on_miss();
    }
  }

  pub fn set_starting_epoch(&mut self) {
    // TODO
  }

  pub fn in_good_mood(&self) -> bool {
    self.has_support_network_for(1.0)
  }

  pub fn set_grateful(&mut self) {
    // TODO
  }

  pub fn set_lonely(&mut self) {
    // TODO
  }

  pub fn set_bored(&mut self) {
    let factor = self.epoch.inactive_for as f32 / self.config.personality.bored_num_epochs as f32;
    if !self.has_support_network_for(factor) {
      LOGGER.log_warning("Automata", "epochs with not activity -> bored");
    } else {
      LOGGER.log_info("Automata", "Unit is grateful instead of bored");
      self.set_grateful();
    }
  }

  pub fn set_sad(&mut self) {
    let factor = self.epoch.inactive_for as f32 / self.config.personality.sad_num_epochs as f32;
    if !self.has_support_network_for(factor) {
      LOGGER.log_warning("Automata", "epochs with not activity -> sad");
    } else {
      LOGGER.log_info("Automata", "Unit is grateful instead of sad");
      self.set_grateful();
    }
  }

  pub fn set_angry(&mut self, factor: f32) {
    if !self.has_support_network_for(factor) {
      LOGGER.log_warning("Automata", "epochs with not activity -> angry");
    } else {
      LOGGER.log_info("Automata", "Unit is grateful instead of angry");
      self.set_grateful();
    }
  }

  pub fn set_excited(&mut self) {
    LOGGER.log_info("Automata", "Unit is excited!");
  }

  pub fn wait_for(&mut self, duration: u32, _sleeping: Option<bool>) {
    self.epoch.track("sleep", Some(duration));
  }

  pub fn is_stale(&mut self) -> bool {
    self.epoch.num_missed > self.config.personality.max_misses_for_recon as u32
  }

  pub fn any_activity(&self) -> bool {
    self.epoch.any_activity
  }

  pub fn next_epoch(&mut self) {
    LOGGER.log_debug("Automata", "Advancing to next epoch");
    
    let was_stale = self.is_stale();
    let did_miss = self.epoch.num_missed;

    self.epoch.next();

    if was_stale {
      let factor = did_miss as f32 / self.config.personality.max_misses_for_recon as f32;
      if factor >= 2.0 {
        self.set_angry(factor);
      } else {
        LOGGER.log_warning("Automata", "epochs with not activity -> lonely");
        self.set_lonely();
      }
    } else if self.epoch.sad_for > 0 {
      let factor = self.epoch.sad_for as f32 / self.config.personality.sad_num_epochs as f32;
      if factor >= 2.0 {
        self.set_angry(factor);
      } else {
        self.set_sad();
      }
    } else if self.epoch.bored_for > 0 {
      self.set_bored();
    } else if self.epoch.active_for >= self.config.personality.excited_num_epochs as u32 {
      self.set_excited();
    } else if self.epoch.active_for >= 5 && self.has_support_network_for(5.0) {
      self.set_grateful();
    }

    // Blindness
    if self.epoch.blind_for >= 5 {
      LOGGER.log_fatal("Automata", "Unit is blind.");
      //self.restart();
      self.epoch.blind_for = 0;
    }
  }

  fn has_support_network_for(&self, factor: f32) -> bool {
    let bond_factor = self.config.personality.bond_encounters_factor as f32;
    let total_encounters = self.epoch.num_peers as f32;
    total_encounters > 0.0 && (bond_factor / total_encounters) >= factor
  }

}