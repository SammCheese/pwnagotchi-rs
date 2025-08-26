use crate::core::{ ai::Epoch, config::config, log::LOGGER, ui::view::View };

pub struct Automata {
  pub epoch: Epoch,
  pub view: View,
}

impl Default for Automata {
  fn default() -> Self {
    let epoch = Epoch::new();
    let view = View::default();
    Self { epoch, view }
  }
}

impl Automata {
  #[must_use]
  pub fn new() -> Self {
    let epoch = Epoch::new();
    let view = View::default();
    Self { epoch, view }
  }

  pub fn on_miss(&mut self) {
    LOGGER.log_info("Automata", "Missed an interaction :(");
    self.epoch.track("miss", None);
  }

  pub fn on_error(&mut self, error: &str) {
    LOGGER.log_error("Automata", error);
    if error.contains("is an unknown BSSID") {
      self.on_miss();
    }
  }

  pub const fn set_starting_epoch(&mut self) {
    // TODO
  }

  pub fn in_good_mood(&self) -> bool {
    self.has_support_network_for(1.0)
  }

  pub fn set_grateful(&mut self) {
    LOGGER.log_info("Automata", "Unit is grateful.");
  }

  pub fn set_lonely(&mut self) {
    LOGGER.log_info("Automata", "Unit is lonely.");
  }

  pub fn set_bored(&mut self) {
    let factor =
      f64::from(self.epoch.inactive_for) / f64::from(config().personality.bored_num_epochs);
    #[allow(clippy::cast_possible_truncation)]
    if self.has_support_network_for(factor as f32) {
      LOGGER.log_info("Automata", "Unit is grateful instead of bored");
      self.set_grateful();
    } else {
      LOGGER.log_warning("Automata", "epochs with not activity -> bored");
    }
  }

  pub fn set_sad(&mut self) {
    let factor =
      f64::from(self.epoch.inactive_for) / f64::from(config().personality.sad_num_epochs);
    #[allow(clippy::cast_possible_truncation)]
    if self.has_support_network_for(factor as f32) {
      LOGGER.log_info("Automata", "Unit is grateful instead of sad");
      self.set_grateful();
    } else {
      LOGGER.log_warning("Automata", "epochs with not activity -> sad");
    }
  }

  pub fn set_angry(&mut self, factor: f32) {
    if self.has_support_network_for(factor) {
      LOGGER.log_info("Automata", "Unit is grateful instead of angry");
      self.set_grateful();
    } else {
      LOGGER.log_warning("Automata", "epochs with not activity -> angry");
    }
  }

  pub fn set_excited(&mut self) {
    LOGGER.log_info("Automata", "Unit is excited!");
  }

  pub fn wait_for(&mut self, duration: u32, sleeping: Option<bool>) {
    self.epoch.track("sleep", Some(duration));
    self.view.wait(duration.into(), sleeping);
  }

  pub fn is_stale(&mut self) -> bool {
    self.epoch.num_missed > config().personality.max_misses_for_recon
  }

  #[must_use]
  pub const fn any_activity(&self) -> bool {
    self.epoch.any_activity
  }

  pub fn next_epoch(&mut self) {
    LOGGER.log_debug("Automata", &format!("Advancing to next epoch {} -> {}", self.epoch.epoch, self.epoch.epoch + 1));

    let was_stale = self.is_stale();
    let did_miss = self.epoch.num_missed;

    self.epoch.next();

    if was_stale {
      let factor = f64::from(did_miss) / f64::from(config().personality.max_misses_for_recon);
      #[allow(clippy::cast_possible_truncation)]
      let factor = factor as f32;
      if factor >= 2.0 {
        self.set_angry(factor);
      } else {
        LOGGER.log_warning("Automata", "epochs with not activity -> lonely");
        self.set_lonely();
      }
    } else if self.epoch.sad_for > 0 {
      let factor = f64::from(self.epoch.sad_for) / f64::from(config().personality.sad_num_epochs);
      if factor >= 2.0 {
        #[allow(clippy::cast_possible_truncation)]
        self.set_angry(factor as f32);
      } else {
        self.set_sad();
      }
    } else if self.epoch.bored_for > 0 {
      self.set_bored();
    } else if self.epoch.active_for >= config().personality.excited_num_epochs {
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
    let bond_factor = f64::from(config().personality.bond_encounters_factor);
    let total_encounters = f64::from(self.epoch.num_peers);
    total_encounters > 0.0 && bond_factor / total_encounters >= factor.into()
  }
}
