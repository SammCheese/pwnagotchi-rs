use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct FaceConfig {
  pub look_r: String,
  pub look_l: String,
  pub look_r_happy: String,
  pub look_l_happy: String,
  pub sleep: String,
  pub sleep2: String,
  pub awake: String,
  pub bored: String,
  pub intense: String,
  pub cool: String,
  pub happy: String,
  pub grateful: String,
  pub excited: String,
  pub motivated: String,
  pub demotivated: String,
  pub smart: String,
  pub lonely: String,
  pub sad: String,
  pub angry: String,
  pub friend: String,
  pub broken: String,
  pub debug: String,
  pub upload: String,
  pub upload1: String,
  pub upload2: String,
  pub png: bool,
  pub position_x: i32,
  pub position_y: i32,
}

impl Default for FaceConfig {
  fn default() -> Self {
    Self {
      look_r: "( ⚆_⚆)".to_string(),
      look_l: "(☉_☉ )".to_string(),
      look_r_happy: "( ◕‿◕)".to_string(),
      look_l_happy: "(◕‿◕ )".to_string(),
      sleep: "(⇀‿‿↼)".to_string(),
      sleep2: "(≖‿‿≖)".to_string(),
      awake: "(◕‿‿◕)".to_string(),
      bored: "(-__-)".to_string(),
      intense: "(°▃▃°)".to_string(),
      cool: "(⌐■_■)".to_string(),
      happy: "(•‿‿•)".to_string(),
      grateful: "(^‿‿^)".to_string(),
      excited: "(ᵔ◡◡ᵔ)".to_string(),
      motivated: "(☼‿‿☼)".to_string(),
      demotivated: "(≖__≖)".to_string(),
      smart: "(✜‿‿✜)".to_string(),
      lonely: "(ب__ب)".to_string(),
      sad: "(╥☁╥ )".to_string(),
      angry: "(-_-')".to_string(),
      friend: "(♥‿‿♥)".to_string(),
      broken: "(☓‿‿☓)".to_string(),
      debug: "(#__#)".to_string(),
      upload: "(1__0)".to_string(),
      upload1: "(1__1)".to_string(),
      upload2: "(0__1)".to_string(),
      png: false,
      position_x: 0,
      position_y: 40,
    }
  }
}
