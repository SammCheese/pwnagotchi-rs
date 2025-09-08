#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Debug)]
pub enum StateValue {
  None,
  Face(FaceType),
  Text(String),
  Number(u64),
  Bool(bool),
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
