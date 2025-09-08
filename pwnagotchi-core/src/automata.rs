use std::sync::Arc;

use parking_lot::Mutex;
use pwnagotchi_shared::{
  config::config,
  log::LOGGER,
  models::net::AccessPoint,
  traits::{automata::AgentObserver, ui::ViewTrait},
};

use crate::ai::Epoch;

pub struct Automata {
  pub epoch: Arc<Mutex<Epoch>>,
  pub view: Arc<dyn ViewTrait + Send + Sync>,
}

impl Automata {
  pub const fn new(epoch: Arc<Mutex<Epoch>>, view: Arc<dyn ViewTrait + Send + Sync>) -> Self {
    Self { epoch, view }
  }

  pub fn clone_as_trait(&self) -> Arc<dyn AgentObserver + Send + Sync> {
    Arc::new(Automata {
      epoch: Arc::clone(&self.epoch),
      view: Arc::clone(&self.view),
    })
  }
}

#[async_trait::async_trait]
impl AgentObserver for Automata {
  fn on_miss(&self, who: &AccessPoint) {
    LOGGER.log_info("Personality", "Missed an interaction :(");
    self.view.on_miss(who);
    self.epoch.lock().track("miss", None);
  }

  fn on_error(&self, who: &AccessPoint, error: &str) {
    LOGGER.log_error("Personality", error);
    if error.contains("is an unknown BSSID") {
      self.on_miss(who);
    }
  }

  fn set_rebooting(&self) {
    self.view.on_rebooting();
  }

  fn set_starting(&self) {
    self.view.on_starting();
  }

  fn set_ready(&self) {
    //plugins.on('ready')
  }

  fn in_good_mood(&self) -> bool {
    self.has_support_network_for(1.0)
  }

  fn set_grateful(&self) {
    self.view.on_grateful();
    LOGGER.log_info("Personality", "Unit is grateful.");
  }

  fn set_lonely(&self) {
    if self.has_support_network_for(1.0) {
      LOGGER.log_info("Personality", "Unit is grateful instead of lonely");
      self.set_grateful();
    } else {
      LOGGER.log_info("Personality", "Unit is lonely.");
      self.view.on_lonely();
    }
  }

  fn set_bored(&self) {
    let factor =
      f64::from(self.epoch.lock().inactive_for) / f64::from(config().personality.bored_num_epochs);
    #[allow(clippy::cast_possible_truncation)]
    if self.has_support_network_for(factor as f32) {
      LOGGER.log_info("Personality", "Unit is grateful instead of bored");
      self.set_grateful();
    } else {
      self.view.on_bored();
      LOGGER.log_warning("Personality", "epochs with not activity -> bored");
    }
  }

  fn set_sad(&self) {
    let factor =
      f64::from(self.epoch.lock().inactive_for) / f64::from(config().personality.sad_num_epochs);

    #[allow(clippy::cast_possible_truncation)]
    if self.has_support_network_for(factor as f32) {
      LOGGER.log_info("Personality", "Unit is grateful instead of sad");
      self.set_grateful();
    } else {
      self.view.on_sad();
      LOGGER.log_warning(
        "Personality",
        format!("{} epochs with no activity -> sad", self.epoch.lock().inactive_for).as_str(),
      );
    }
  }

  fn set_angry(&self, factor: f32) {
    if self.has_support_network_for(factor) {
      LOGGER.log_info("Personality", "Unit is grateful instead of angry");
      self.set_grateful();
    } else {
      self.view.on_angry();
      LOGGER.log_warning(
        "Personality",
        format!("{} epochs with no activity -> angry", self.epoch.lock().inactive_for).as_str(),
      );
    }
  }

  fn set_excited(&self) {
    LOGGER.log_info(
      "Personality",
      format!("{} epochs with activity -> excited", self.epoch.lock().active_for).as_str(),
    );
    self.view.on_excited();
  }

  async fn wait_for(&self, duration: u32, sleeping: Option<bool>) {
    let sleeping = sleeping.unwrap_or(true);
    {
      self.epoch.lock().track("sleep", Some(duration));
    }
    self.view.wait(duration.into(), sleeping, &self.clone_as_trait()).await;
  }

  fn is_stale(&self) -> bool {
    self.epoch.lock().num_missed > config().personality.max_misses_for_recon
  }

  fn any_activity(&self) -> bool {
    self.epoch.lock().any_activity
  }

  fn next_epoch(&self) {
    let was_stale = self.is_stale();

    let (_epoch_num, did_miss, sad_for, bored_for, active_for, blind_for, was_stale) = {
      let mut epoch = self.epoch.lock();

      LOGGER.log_debug(
        "Epoch",
        &format!("Advancing to next epoch {} -> {}", epoch.epoch, epoch.epoch + 1),
      );

      let did_miss = epoch.num_missed;
      epoch.next();

      (
        epoch.epoch,
        did_miss,
        epoch.sad_for,
        epoch.bored_for,
        epoch.active_for,
        epoch.blind_for,
        was_stale,
      )
    };

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
    } else if sad_for > 0 {
      let factor = f64::from(sad_for) / f64::from(config().personality.sad_num_epochs);

      if factor >= 2.0 {
        #[allow(clippy::cast_possible_truncation)]
        self.set_angry(factor as f32);
      } else {
        self.set_sad();
      }
    } else if bored_for > 0 {
      self.set_bored();
    } else if active_for >= config().personality.excited_num_epochs {
      self.set_excited();
    } else if active_for >= 5 && self.has_support_network_for(5.0) {
      self.set_grateful();
    }

    // Blindness
    if blind_for >= 5 {
      LOGGER.log_fatal(
        "Personality",
        format!("{blind_for} epochs without visible access points -> we are blind!").as_str(),
      );

      //self.restart();
      self.epoch.lock().blind_for = 0;
    }
  }

  fn has_support_network_for(&self, factor: f32) -> bool {
    let bond_factor = f64::from(config().personality.bond_encounters_factor);
    let total_encounters = f64::from(self.epoch.lock().num_peers);

    total_encounters > 0.0 && (bond_factor / total_encounters) >= factor.into()
  }
}
