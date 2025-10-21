use std::error::Error;

use async_trait::async_trait;
use serde::Serialize;

use crate::types::events::EventPayload;

#[async_trait]
pub trait EventBus: Send + Sync {
  async fn emit_payload(
    &self,
    event: &str,
    payload: EventPayload,
  ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

/// Emit an event with automatic serialization (async version).
/// Use this in async contexts.
pub async fn emit_serialized<B, T>(
  bus: &B,
  event: &str,
  value: &T,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
  B: EventBus + ?Sized,
  T: Serialize + Send + Sync,
{
  let payload =
    EventPayload::new(value).map_err(|e| -> Box<dyn Error + Send + Sync> { e.into() })?;
  bus.emit_payload(event, payload).await
}
