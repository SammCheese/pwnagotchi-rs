use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct FaceConfig {
  pub look_r: Cow<'static, str>,
  pub look_l: Cow<'static, str>,
  pub look_r_happy: Cow<'static, str>,
  pub look_l_happy: Cow<'static, str>,
  pub sleep: Cow<'static, str>,
  pub sleep2: Cow<'static, str>,
  pub awake: Cow<'static, str>,
  pub bored: Cow<'static, str>,
  pub intense: Cow<'static, str>,
  pub cool: Cow<'static, str>,
  pub happy: Cow<'static, str>,
  pub grateful: Cow<'static, str>,
  pub excited: Cow<'static, str>,
  pub motivated: Cow<'static, str>,
  pub demotivated: Cow<'static, str>,
  pub smart: Cow<'static, str>,
  pub lonely: Cow<'static, str>,
  pub sad: Cow<'static, str>,
  pub angry: Cow<'static, str>,
  pub friend: Cow<'static, str>,
  pub broken: Cow<'static, str>,
  pub debug: Cow<'static, str>,
  pub upload: Cow<'static, str>,
  pub upload1: Cow<'static, str>,
  pub upload2: Cow<'static, str>,
  pub png: bool,
  pub position_x: i32,
  pub position_y: i32,
}

impl Default for FaceConfig {
  fn default() -> Self {
    Self {
      look_r: "( ⚆_⚆)".into(),
      look_l: "(☉_☉ )".into(),
      look_r_happy: "( ◕‿◕)".into(),
      look_l_happy: "(◕‿◕ )".into(),
      sleep: "(⇀‿‿↼)".into(),
      sleep2: "(≖‿‿≖)".into(),
      awake: "(◕‿‿◕)".into(),
      bored: "(-__-)".into(),
      intense: "(°▃▃°)".into(),
      cool: "(⌐■_■)".into(),
      happy: "(•‿‿•)".into(),
      grateful: "(^‿‿^)".into(),
      excited: "(ᵔ◡◡ᵔ)".into(),
      motivated: "(☼‿‿☼)".into(),
      demotivated: "(≖__≖)".into(),
      smart: "(✜‿‿✜)".into(),
      lonely: "(ب__ب)".into(),
      sad: "(╥☁╥ )".into(),
      angry: "(-_-')".into(),
      friend: "(♥‿‿♥)".into(),
      broken: "(☓‿‿☓)".into(),
      debug: "(#__#)".into(),
      upload: "(1__0)".into(),
      upload1: "(1__1)".into(),
      upload2: "(0__1)".into(),
      png: false,
      position_x: 0,
      position_y: 40,
    }
  }
}
