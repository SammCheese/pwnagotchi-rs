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
pub struct UIWebConfig {
  pub enabled: bool,
  pub address: String,
  pub port: u16,
  pub username: String,
  pub password: String,
  pub origin: String,
  pub on_frame: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UIDisplayConfig {
  pub enabled: bool,
  pub rotation: u32,
  pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UIFontConfig {
  pub size_offset: u32,
  pub name: String,
}

impl Default for UIConfig {
  fn default() -> Self {
    Self {
      inverted: false,
      fps: 0.0,
      cursor: true,
      web: UIWebConfig {
        enabled: true,
        address: "127.0.0.1".into(),
        port: 8080,
        username: "user".into(),
        password: "pass".into(),
        origin: "http://localhost".into(),
        on_frame: "console.log('frame')".into(),
      },
      display: UIDisplayConfig {
        enabled: false,
        rotation: 180,
        r#type: "waveshare_v4".into(),
      },
      font: UIFontConfig {
        size_offset: 0,
        name: "DejaVuSansMono".into(),
      },
    }
  }
}
