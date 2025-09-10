#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum StateValue {
  Text(String),
  Number(f64),
  None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FaceType {
  LookR,
  LookL,
  LookRHappy,
  LookLHappy,
  Sleep,
  Sleep2,
  Awake,
  Bored,
  Intense,
  Cool,
  Happy,
  Grateful,
  Excited,
  Motivated,
  Demotivated,
  Smart,
  Lonely,
  Sad,
  Angry,
  Friend,
  Broken,
  Debug,
  Upload,
  Upload1,
  Upload2,
}
