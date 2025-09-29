use std::{fmt::Write, sync::Arc};

use anyhow::Result;
use pwnagotchi_shared::{
  config::config,
  sessions::manager::SessionManager,
  traits::{
    general::{Component, CoreModules, Dependencies},
    ui::{UIRefresher, ViewTrait},
  },
  utils::{
    agent::get_aps_on_channel,
    general::{format_duration_human, total_unique_handshakes},
  },
};
use tokio::task::JoinHandle;

pub struct RefresherComponent {
  refresher: Option<Arc<Refresher>>,
}

impl Default for RefresherComponent {
  fn default() -> Self {
    Self::new()
  }
}

impl RefresherComponent {
  #[must_use]
  pub const fn new() -> Self {
    Self { refresher: None }
  }
}

impl Dependencies for RefresherComponent {
  fn name(&self) -> &'static str {
    "UIRefresherComponent"
  }

  fn dependencies(&self) -> &[&str] {
    &["SessionManager", "View"]
  }
}

#[async_trait::async_trait]
impl Component for RefresherComponent {
  async fn init(&mut self, _ctx: &CoreModules) -> Result<()> {
    let (sm, view) = (&_ctx.session_manager, &_ctx.view);
    let sm = Arc::clone(sm);
    let view = Arc::clone(view);
    self.refresher = Some(Arc::new(Refresher::new(sm, view)));

    Ok(())
  }

  async fn start(&self) -> Result<Option<JoinHandle<()>>> {
    if let Some(refresher) = &self.refresher {
      let refresher = Arc::clone(refresher);

      let handle = tokio::spawn(async move {
        refresher.start().await;
      });
      return Ok(Some(handle));
    }
    Ok(None)
  }
}

impl Dependencies for Refresher {
  fn name(&self) -> &'static str {
    "UIRefresher"
  }

  fn dependencies(&self) -> &[&'static str] {
    &["SessionManager", "View"]
  }
}

pub struct Refresher {
  sm: Arc<SessionManager>,
  view: Arc<dyn ViewTrait + Send + Sync>,
}

#[async_trait::async_trait]
impl UIRefresher for Refresher {
  async fn start(&self) {
    self.start_sessionfetcher().await;
  }
}

impl Refresher {
  pub const fn new(sm: Arc<SessionManager>, view: Arc<dyn ViewTrait + Send + Sync>) -> Self {
    Self { sm, view }
  }

  pub async fn start_sessionfetcher(&self) {
    loop {
      self.update_uptime();
      self.update_aps();
      self.update_handshakes();
      tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
  }

  fn update_handshakes(&self) {
    let handshakes_path = config().bettercap.handshakes.to_string();
    let total = total_unique_handshakes(&handshakes_path);
    let session = self.sm.get_session();
    let state = &session.read().state;

    let (current, last_pwned) = {
      let current = state.handshakes.len();
      let last_pwned = state.last_pwned.clone();
      (current, last_pwned)
    };

    let mut text = format!("{current} ({total})");
    if let Some(last_pwned) = last_pwned {
      let _ = write!(text, " [{last_pwned}]");
    }

    self.view.set("handshakes", text);
  }

  fn update_aps(&self) {
    let session = self.sm.get_session();
    let state = &session.read().state;
    let (ap_data, sta_data) = {
      let tot_aps = state.access_points.len();
      let tot_stas: usize = state.access_points.iter().map(|ap| ap.clients.len()).sum();
      let on_channel = state.current_channel != 0;
      let aps_on_channel =
        if on_channel { get_aps_on_channel(&session, state.current_channel) } else { Vec::new() };

      let ap_data = if on_channel {
        let aps_on_channel = get_aps_on_channel(&session, state.current_channel);
        format!("{} ({})", aps_on_channel.len(), tot_aps)
      } else {
        tot_aps.to_string()
      };

      let sta_data = if on_channel {
        let stas_on_channel: usize = aps_on_channel.iter().map(|ap| ap.clients.len()).sum();
        format!("{stas_on_channel} ({tot_stas})")
      } else {
        tot_stas.to_string()
      };
      (ap_data, sta_data)
    };

    self.view.set("aps", ap_data);
    self.view.set("sta", sta_data);
  }

  fn update_uptime(&self) {
    let started_at = self.sm.get_session().read().started_at;
    let now = std::time::SystemTime::now();
    let text = format_duration_human(now.duration_since(started_at).unwrap_or_default());
    self.view.set("uptime", text);
  }
}
