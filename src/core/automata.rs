use crate::core::{
  ai::Epoch,
  config::config,
  log::LOGGER,
  models::net::AccessPoint,
  ui::{old::hw::base::get_display_from_config, view::View},
};

pub struct Automata {
  pub epoch: Epoch,
  pub view: View,
}

impl Default for Automata {
  fn default() -> Self {
    let epoch = Epoch::new();
    let view = View::new(get_display_from_config());
    Self { epoch, view }
  }
}

impl Automata {
  #[must_use]
  pub fn new() -> Self {
    let epoch = Epoch::new();
    let view = View::new(get_display_from_config());
    Self { epoch, view }
  }

  pub fn on_miss(&mut self, who: &AccessPoint) {
    LOGGER.log_info("Personality", "Missed an interaction :(");
    self.view.on_miss(who);
    self.epoch.track("miss", None);
  }

  pub fn on_error(&mut self, who: &AccessPoint, error: &str) {
    LOGGER.log_error("Personality", error);
    if error.contains("is an unknown BSSID") {
      self.on_miss(who);
    }
  }

  pub fn set_starting(&self) {
    self.view.on_starting();
  }

  pub const fn set_ready(&self) {
    //plugins.on('ready')
  }

  pub fn in_good_mood(&self) -> bool {
    self.has_support_network_for(1.0)
  }

  pub fn set_grateful(&mut self) {
    self.view.on_grateful();
    LOGGER.log_info("Personality", "Unit is grateful.");
  }

  pub fn set_lonely(&mut self) {
    if self.has_support_network_for(1.0) {
      LOGGER.log_info("Personality", "Unit is grateful instead of lonely");
      self.set_grateful();
    } else {
      LOGGER.log_info("Personality", "Unit is lonely.");
      self.view.on_lonely();
    }
  }

  pub fn set_bored(&mut self) {
    let factor =
      f64::from(self.epoch.inactive_for) / f64::from(config().personality.bored_num_epochs);
    #[allow(clippy::cast_possible_truncation)]
    if self.has_support_network_for(factor as f32) {
      LOGGER.log_info("Personality", "Unit is grateful instead of bored");
      self.set_grateful();
    } else {
      self.view.on_bored();
      LOGGER.log_warning("Personality", "epochs with not activity -> bored");
    }
  }

  pub fn set_sad(&mut self) {
    let factor =
      f64::from(self.epoch.inactive_for) / f64::from(config().personality.sad_num_epochs);
    #[allow(clippy::cast_possible_truncation)]
    if self.has_support_network_for(factor as f32) {
      LOGGER.log_info("Personality", "Unit is grateful instead of sad");
      self.set_grateful();
    } else {
      self.view.on_sad();
      LOGGER.log_warning(
        "Personality",
        format!("{} epochs with no activity -> sad", self.epoch.inactive_for).as_str(),
      );
    }
  }

  pub fn set_angry(&mut self, factor: f32) {
    if self.has_support_network_for(factor) {
      LOGGER.log_info("Personality", "Unit is grateful instead of angry");
      self.set_grateful();
    } else {
      self.view.on_angry();
      LOGGER.log_warning(
        "Personality",
        format!(
          "{} epochs with no activity -> angry",
          self.epoch.inactive_for
        )
        .as_str(),
      );
    }
  }

  pub fn set_excited(&mut self) {
    LOGGER.log_info(
      "Personality",
      format!("{} epochs with activity -> excited", self.epoch.active_for).as_str(),
    );
    self.view.on_excited();
  }

  pub async fn wait_for(&mut self, duration: u32, sleeping: Option<bool>) {
    let sleeping = sleeping.unwrap_or(true);
    self.view.wait(duration.into(), sleeping, self).await;
    self.epoch.track("sleep", Some(duration));
  }

  pub fn is_stale(&mut self) -> bool {
    self.epoch.num_missed > config().personality.max_misses_for_recon
  }

  pub const fn any_activity(&self) -> bool {
    self.epoch.any_activity
  }

  pub fn next_epoch(&mut self) {
    LOGGER.log_debug(
      "Epoch",
      &format!(
        "Advancing to next epoch {} -> {}",
        self.epoch.epoch,
        self.epoch.epoch + 1
      ),
    );

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
        LOGGER.log_warning(
          "Personality",
          format!("agent missed {did_miss} interactions -> lonely").as_str(),
        );
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
      LOGGER.log_fatal(
        "Personality",
        format!(
          "{} epochs without visible access points -> we are blind!",
          self.epoch.blind_for
        )
        .as_str(),
      );
      //self.restart();
      self.epoch.blind_for = 0;
    }
  }

  fn has_support_network_for(&self, factor: f32) -> bool {
    let bond_factor = f64::from(config().personality.bond_encounters_factor);
    let total_encounters = f64::from(self.epoch.num_peers);
    total_encounters > 0.0 && (bond_factor / total_encounters) >= factor.into()
  }
}
