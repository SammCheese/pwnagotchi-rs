use std::{
  error::Error,
  fmt::{Display, Formatter, Result as FmtResult},
  sync::Arc,
};

use futures::future::BoxFuture;
use pwnagotchi_shared::types::events::EventPayload;

pub type EventHandler =
  Arc<dyn Fn(&EventPayload) -> Result<(), Box<dyn Error + Send + Sync>> + Send + Sync>;

pub type AsyncEventHandler = Arc<
  dyn Fn(EventPayload) -> BoxFuture<'static, Result<(), Box<dyn Error + Send + Sync>>>
    + Send
    + Sync,
>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventError {
  ListenerFailed { plugin: String, event: String, message: String },
  UnregisterFailed(String),
}

impl Display for EventError {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    match self {
      EventError::ListenerFailed { plugin, event, message } => {
        write!(f, "Listener for event '{event}' in plugin '{plugin}' failed: {message}")
      }
      EventError::UnregisterFailed(name) => {
        write!(f, "Failed to unregister listener '{name}'")
      }
    }
  }
}

impl Error for EventError {}

pub trait DynamicEventAPITrait {
  fn registered(&self) -> &[(String, u64)];

  fn register_listener(&mut self, event: &str, handler: EventHandler) -> Result<u64, EventError>;

  fn register_async_listener(
    &mut self,
    event: &str,
    handler: AsyncEventHandler,
  ) -> Result<u64, EventError>;

  fn cleanup(&mut self) -> Result<(), EventError>;

  fn unregister(&mut self, listener_id: &u64) -> Result<(), EventError>;
}
