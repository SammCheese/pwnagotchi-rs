use std::fmt;

use crate::{config::config, types::ui::FaceType};

impl fmt::Display for FaceType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let faces = &config().faces;
    let s = match self {
      FaceType::LookR => &faces.look_r,
      FaceType::LookL => &faces.look_l,
      FaceType::LookRHappy => &faces.look_r_happy,
      FaceType::LookLHappy => &faces.look_l_happy,
      FaceType::Sleep => &faces.sleep,
      FaceType::Sleep2 => &faces.sleep2,
      FaceType::Awake => &faces.awake,
      FaceType::Bored => &faces.bored,
      FaceType::Intense => &faces.intense,
      FaceType::Cool => &faces.cool,
      FaceType::Happy => &faces.happy,
      FaceType::Grateful => &faces.grateful,
      FaceType::Excited => &faces.excited,
      FaceType::Motivated => &faces.motivated,
      FaceType::Demotivated => &faces.demotivated,
      FaceType::Smart => &faces.smart,
      FaceType::Lonely => &faces.lonely,
      FaceType::Sad => &faces.sad,
      FaceType::Angry => &faces.angry,
      FaceType::Friend => &faces.friend,
      FaceType::Broken => &faces.broken,
      FaceType::Debug => &faces.debug,
      FaceType::Upload => &faces.upload,
      FaceType::Upload1 => &faces.upload1,
      FaceType::Upload2 => &faces.upload2,
    };
    write!(f, "{}", s)
  }
}
