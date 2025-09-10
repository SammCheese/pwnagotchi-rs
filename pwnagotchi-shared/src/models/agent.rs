use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunningMode {
  Auto,
  Manual,
  Ai,
  Custom,
}

impl Display for RunningMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      RunningMode::Auto => "AUTO",
      RunningMode::Manual => "MANU",
      RunningMode::Ai => "AI",
      RunningMode::Custom => "CUST",
    };
    write!(f, "{}", s)
  }
}
