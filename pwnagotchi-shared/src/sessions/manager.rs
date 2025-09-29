use std::sync::Arc;

use parking_lot::RwLock;
use tokio::sync::broadcast;

use crate::{
  sessions::{lastsession::LastSession, session::Session},
  traits::general::CoreModule,
};

impl CoreModule for SessionManager {
  fn name(&self) -> &'static str {
    "SessionManager"
  }
}

pub struct SessionManager {
  current: Arc<RwLock<Session>>,
  last: Arc<RwLock<LastSession>>,
  notifier: broadcast::Sender<()>,
}

impl Default for SessionManager {
  fn default() -> Self {
    Self::new()
  }
}

impl SessionManager {
  pub fn new() -> Self {
    let (tx, _rx) = broadcast::channel(16);
    let current = Arc::new(RwLock::new(Session::new()));
    let last = Arc::new(RwLock::new(LastSession::new()));

    Self { current, last, notifier: tx }
  }

  pub fn set_session(&self, new_session: Session) {
    {
      let mut current = self.current.write();
      *current = new_session;
    }
    let _ = self.notifier.send(());
  }

  //pub async fn set_last_session(&self, last: Option<LastSession>) {
  //  let mut ls = self.last.write().await;
  //  *ls = last.map(Arc::new);
  //}

  pub fn get_last_session(&self) -> Arc<RwLock<LastSession>> {
    Arc::clone(&self.last)
  }

  pub fn get_session(&self) -> Arc<RwLock<Session>> {
    Arc::clone(&self.current)
  }

  pub fn subscribe(&self) -> broadcast::Receiver<()> {
    self.notifier.subscribe()
  }

  /*async fn save_recovery_data(&self) {
    LOGGER.log_warning("SessionManager", "Saving recovery data...");
    let session = self.current.read().await;
    let state = session.state.read().clone();
    let data = serde_json::json!({
        "started_at": session.started_at,
        "history": state.history,
        "handshakes": state.handshakes,
        "last_pwned": state.last_pwned,
      }
    );
    drop(state);
    drop(session);
    tokio::fs::write(RECOVERY_FILE, data.to_string()).await.unwrap_or_else(|e| {
      LOGGER.log_error("SessionManager", &format!("Failed to write recovery data: {e}"));
    });
  }

  async fn load_recovery_data(&self) {
    if let Ok(data) = tokio::fs::read_to_string(RECOVERY_FILE).await {
      if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
        let started_at = json.get("started_at").and_then(|v| v.as_u64()).unwrap_or(0);
        let history = json
          .get("history")
          .and_then(|v| v.as_array())
          .map(|arr| {
            arr
              .iter()
              .filter_map(|v| v.as_str().map(|s| s.to_string()))
              .collect::<Vec<String>>()
          })
          .unwrap_or_default();
        let handshakes = json
          .get("handshakes")
          .and_then(|v| v.as_array())
          .map(|arr| {
            arr
              .iter()
              .filter_map(|v| v.as_str().map(|s| s.to_string()))
              .collect::<Vec<String>>()
          })
          .unwrap_or_default();
        let last_pwned = json.get("last_pwned").and_then(|v| v.as_str()).map(|s| s.to_string());

        let mut session = self.current.write().await;
        let mut state = session.state.write();
        state.history = history.into_iter().map(|k| (k, 0u32)).collect();
        state.handshakes = handshakes.into_iter().map(|h| (h, Handshake::default())).collect();
        state.last_pwned = last_pwned.map(|s| std::borrow::Cow::Owned(s));
        drop(state);
        session.started_at = SystemTime::UNIX_EPOCH
          .checked_add(std::time::Duration::from_secs(started_at))
          .unwrap_or(SystemTime::now());
        drop(session);

        LOGGER.log_warning("SessionManager", "Recovery data loaded.");
      }
    }
  }*/
}
