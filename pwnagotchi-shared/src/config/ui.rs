use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct UIConfig {
  pub inverted: bool,
  pub fps: f32,
  pub cursor: bool,

  pub web: UIWebConfig,
  pub display: UIDisplayConfig,
  pub font: UIFontConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct UIWebConfig {
  pub enabled: bool,
  pub address: Cow<'static, str>,
  pub port: u16,
  pub username: Cow<'static, str>,
  pub password: Cow<'static, str>,
  pub origin: Cow<'static, str>,
  pub on_frame: Cow<'static, str>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct UIDisplayConfig {
  pub enabled: bool,
  pub rotation: u32,
  pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct UIFontConfig {
  pub size_offset: u32,
  pub name: String,
}

impl Default for UIWebConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      address: "127.0.0.1".into(),
      port: 8080,
      username: "".into(),
      password: "".into(),
      origin: "".into(),
      on_frame: "".into(),
    }
  }
}

impl Default for UIDisplayConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      rotation: 180,
      r#type: "waveshare_v4".into(),
    }
  }
}

impl Default for UIFontConfig {
  fn default() -> Self {
    Self {
      size_offset: 0,
      name: "DejaVuSansMono".into(),
    }
  }
}

impl Default for UIConfig {
  fn default() -> Self {
    Self {
      inverted: false,
      fps: 0.0,
      cursor: true,
      web: UIWebConfig::default(),
      display: UIDisplayConfig::default(),
      font: UIFontConfig::default(),
    }
  }
}
