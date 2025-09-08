use std::{fmt::Write, sync::Arc};

use pwnagotchi_shared::{
  config::config,
  sessions::manager::SessionManager,
  traits::ui::ViewTrait,
  types::ui::StateValue,
  utils::{
    agent::get_aps_on_channel,
    general::{format_duration_human, total_unique_handshakes},
  },
};

pub async fn start_sessionfetcher(
  sm: &Arc<SessionManager>,
  view: &Arc<dyn ViewTrait + Send + Sync>,
) {
  let view = Arc::clone(view);
  let sm = Arc::clone(sm);
  loop {
    update_uptime(&sm, &view).await;
    update_aps(&sm, &view).await;
    update_handshakes(&sm, &view).await;
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
  }
}

async fn update_handshakes(sm: &Arc<SessionManager>, view: &Arc<dyn ViewTrait + Send + Sync>) {
  let handshakes_path = config().bettercap.handshakes.to_string();
  let total = total_unique_handshakes(&handshakes_path);
  let session = sm.get_session().await;
  let state = session.state.read();

  let (current, last_pwned) = {
    let current = state.handshakes.len();
    let last_pwned = state.last_pwned.clone();
    drop(state);
    (current, last_pwned)
  };

  let mut text = format!("{current} ({total})");
  if let Some(last_pwned) = last_pwned {
    let _ = write!(text, " [{last_pwned}]");
  }

  view.set("shakes", StateValue::Text(text));
}

async fn update_aps(sm: &Arc<SessionManager>, view: &Arc<dyn ViewTrait + Send + Sync>) {
  let session = sm.get_session().await;
  let state = session.state.read();
  let (ap_data, sta_data) = {
    let tot_aps = state.access_points.len();
    let tot_stas: usize = state.access_points.iter().map(|ap| ap.clients.len()).sum();
    let on_channel = state.current_channel != 0;
    let aps_on_channel =
      if on_channel { get_aps_on_channel(&session, state.current_channel) } else { Vec::new() };

    let ap_data = if on_channel {
      let aps_on_channel = get_aps_on_channel(&session, state.current_channel);
      drop(state);
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

  view.set("aps", StateValue::Text(ap_data));
  view.set("sta", StateValue::Text(sta_data));
}

async fn update_uptime(sm: &Arc<SessionManager>, view: &Arc<dyn ViewTrait + Send + Sync>) {
  let started_at = sm.get_session().await.started_at;
  let now = std::time::SystemTime::now();
  let text = format_duration_human(now.duration_since(started_at).unwrap_or_default());
  view.set("uptime", StateValue::Text(text));
}
