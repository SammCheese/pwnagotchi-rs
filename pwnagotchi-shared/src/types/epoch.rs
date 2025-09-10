#[derive(Debug, Clone, Copy)]
pub enum Activity {
  Deauth,
  Association,
  Miss,
  Hop,
  Handshake,
  Sleep,
}
