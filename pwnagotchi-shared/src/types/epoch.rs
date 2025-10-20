use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Activity {
  Deauth,
  Association,
  Miss,
  Hop,
  Handshake,
  Sleep,
}
