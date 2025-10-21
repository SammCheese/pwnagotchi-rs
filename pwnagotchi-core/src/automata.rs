use std::sync::Arc;

use anyhow::Result;
use parking_lot::RwLock;
use pwnagotchi_shared::{
  config::config,
  logger::LOGGER,
  models::net::AccessPoint,
  traits::{
    automata::AutomataTrait,
    epoch::Epoch,
    events::EventBus,
    general::{Component, CoreModule, CoreModules, Dependencies},
    ui::ViewTrait,
  },
  types::{epoch::Activity, events::EventPayload},
  utils::general::has_support_network_for,
};
use tokio::task::JoinHandle;

pub struct AutomataComponent {}

impl Dependencies for AutomataComponent {
  fn name(&self) -> &'static str {
    "AutomataComponent"
  }

  fn dependencies(&self) -> &[&str] {
    &["Epoch", "View"]
  }
}

#[async_trait::async_trait]
impl Component for AutomataComponent {
  async fn init(&mut self, _ctx: &CoreModules) -> Result<()> {
    Ok(())
  }

  async fn start(&self) -> Result<Option<JoinHandle<()>>> {
    Ok(None)
  }
}

impl Default for AutomataComponent {
  fn default() -> Self {
    Self::new()
  }
}

impl AutomataComponent {
  pub const fn new() -> Self {
    Self {}
  }
}

impl CoreModule for Automata {
  fn name(&self) -> &'static str {
    "Automata"
  }

  fn dependencies(&self) -> &[&'static str] {
    &["Epoch", "View"]
  }
}

pub struct Automata {
  pub epoch: Arc<RwLock<Epoch>>,
  pub eventbus: Arc<dyn EventBus>,
  pub view: Arc<dyn ViewTrait + Send + Sync>,
}

impl Automata {
  pub const fn new(
    epoch: Arc<RwLock<Epoch>>,
    eventbus: Arc<dyn EventBus>,
    view: Arc<dyn ViewTrait + Send + Sync>,
  ) -> Self {
    Self { epoch, eventbus, view }
  }
}

#[async_trait::async_trait]
impl AutomataTrait for Automata {
  fn on_miss(&self, who: &AccessPoint) {
    LOGGER.log_info("Personality", "Missed an interaction :(");
    self.view.on_miss(who);
    self.epoch.write().track(Activity::Miss, None);
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
    let event = Arc::clone(&self.eventbus);
    tokio::task::spawn(async move {
      let _ = event.emit_payload("ready", EventPayload::empty()).await;
    });
  }

  fn in_good_mood(&self) -> bool {
    has_support_network_for(1.0, &self.epoch)
  }

  fn set_grateful(&self) {
    self.view.on_grateful();
    LOGGER.log_info("Personality", "Unit is grateful.");
  }

  fn set_lonely(&self) {
    if has_support_network_for(1.0, &self.epoch) {
      LOGGER.log_info("Personality", "Unit is grateful instead of lonely");
      self.set_grateful();
    } else {
      LOGGER.log_info("Personality", "Unit is lonely.");
      self.view.on_lonely();
    }
  }

  fn set_bored(&self) {
    let factor =
      f64::from(self.epoch.read().inactive_for) / f64::from(config().personality.bored_num_epochs);
    #[allow(clippy::cast_possible_truncation)]
    if has_support_network_for(factor as f32, &self.epoch) {
      LOGGER.log_info("Personality", "Unit is grateful instead of bored");
      self.set_grateful();
    } else {
      self.view.on_bored();
      LOGGER.log_warning("Personality", "epochs with not activity -> bored");
    }
  }

  fn set_sad(&self) {
    let factor =
      f64::from(self.epoch.read().inactive_for) / f64::from(config().personality.sad_num_epochs);

    #[allow(clippy::cast_possible_truncation)]
    if has_support_network_for(factor as f32, &self.epoch) {
      LOGGER.log_info("Personality", "Unit is grateful instead of sad");
      self.set_grateful();
    } else {
      self.view.on_sad();
      LOGGER.log_warning(
        "Personality",
        format!("{} epochs with no activity -> sad", self.epoch.read().inactive_for).as_str(),
      );
    }
  }

  fn set_angry(&self, factor: f32) {
    if has_support_network_for(factor, &self.epoch) {
      LOGGER.log_info("Personality", "Unit is grateful instead of angry");
      self.set_grateful();
    } else {
      self.view.on_angry();
      LOGGER.log_warning(
        "Personality",
        format!("{} epochs with no activity -> angry", self.epoch.read().inactive_for).as_str(),
      );
    }
  }

  fn set_excited(&self) {
    LOGGER.log_info(
      "Personality",
      format!("{} epochs with activity -> excited", self.epoch.read().active_for).as_str(),
    );
    self.view.on_excited();
  }

  async fn wait_for(&self, duration: u32, sleeping: Option<bool>) {
    let sleeping = sleeping.unwrap_or(true);
    self.epoch.write().track(Activity::Sleep, Some(duration));
    self.view.wait(duration.into(), sleeping).await;
  }

  fn is_stale(&self) -> bool {
    self.epoch.read().num_missed > config().personality.max_misses_for_recon
  }

  fn any_activity(&self) -> bool {
    self.epoch.read().any_activity
  }

  fn next_epoch(&self) {
    let was_stale = self.is_stale();
    let epoch_num = self.epoch.read().epoch;

    LOGGER
      .log_info("Epoch", &format!("Advancing to next epoch {} -> {}", epoch_num, epoch_num + 1));

    self.epoch.write().next();

    let (did_miss, sad_for, bored_for, active_for, blind_for, was_stale) = {
      let e = self.epoch.read();
      (e.num_missed, e.sad_for, e.bored_for, e.active_for, e.blind_for, was_stale)
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
    } else if active_for >= 5 && has_support_network_for(5.0, &self.epoch) {
      self.set_grateful();
    }

    // Blindness
    if blind_for >= 5 {
      LOGGER.log_fatal(
        "Personality",
        format!("{blind_for} epochs without visible access points -> we are blind!").as_str(),
      );

      //self.restart();
      self.epoch.write().blind_for = 0;
    }
  }
}
