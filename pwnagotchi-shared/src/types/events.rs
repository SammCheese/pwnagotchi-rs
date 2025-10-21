use std::sync::Arc;

use anyhow::{Result, anyhow};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone, Debug)]
pub struct EventPayload {
  type_name: &'static str,
  bytes: Arc<Vec<u8>>,
}

impl EventPayload {
  pub fn new<T>(value: &T) -> Result<Self>
  where
    T: Serialize,
  {
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(value, config)
      .map_err(|e| anyhow!("failed to serialize event payload: {e}"))?;

    Ok(Self {
      type_name: std::any::type_name::<T>(),
      bytes: Arc::new(bytes),
    })
  }

  pub fn empty() -> Self {
    Self {
      type_name: std::any::type_name::<()>(),
      bytes: Arc::new(Vec::new()),
    }
  }

  #[must_use]
  pub fn type_name(&self) -> &'static str {
    self.type_name
  }

  #[must_use]
  pub fn as_bytes(&self) -> &[u8] {
    self.bytes.as_slice()
  }

  pub fn deserialize<T>(&self) -> Result<T>
  where
    T: DeserializeOwned,
  {
    let config = bincode::config::standard();
    let (value, _): (T, usize) = bincode::serde::decode_from_slice(self.as_bytes(), config)
      .map_err(|e| anyhow!("failed to deserialize event payload: {e}"))?;
    Ok(value)
  }
}
