use std::sync::Arc;

use tokio::sync::{RwLock, broadcast};

use crate::core::sessions::{lastsession::LastSession, session::Session};

pub struct SessionManager {
  current: Arc<RwLock<Arc<Session>>>,
  last: Arc<RwLock<Option<Arc<LastSession>>>>,
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
    Self {
      // What the fuck
      current: Arc::new(RwLock::new(Arc::new(Session::new()))),
      last: Arc::new(RwLock::new(Some(Arc::new(LastSession::new())))),
      notifier: tx,
    }
  }

  pub async fn set_session(&self, new_session: Session) {
    {
      let mut current = self.current.write().await;
      *current = Arc::new(new_session);
    }
    let _ = self.notifier.send(());
  }

  //pub async fn set_last_session(&self, last: Option<LastSession>) {
  //  let mut ls = self.last.write().await;
  //  *ls = last.map(Arc::new);
  //}

  pub async fn get_last_session(&self) -> Option<Arc<LastSession>> {
    self.last.read().await.clone()
  }

  pub async fn get_session(&self) -> Arc<Session> {
    self.current.read().await.clone()
  }

  pub fn subscribe(&self) -> broadcast::Receiver<()> {
    self.notifier.subscribe()
  }
}
