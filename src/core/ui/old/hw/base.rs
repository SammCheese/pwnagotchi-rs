use std::sync::Arc;

use image::RgbaImage;

use crate::core::{config::config, ui::old::hw::waveshare2in13b_v4::Waveshare2in13bV4};

pub trait DisplayTrait: Send + Sync {
  fn layout(&self) -> &Layout;
  fn initialize(&self);
  fn render(&self, canvas: &mut RgbaImage);
  fn clear(&self);
}

#[derive(Debug, Clone)]
pub struct Layout {
  pub width: u32,
  pub height: u32,
  pub face: (u32, u32),
  pub name: (u32, u32),
  pub channel: (u32, u32),
  pub aps: (u32, u32),
  pub uptime: (u32, u32),
  pub line1: ((f32, f32), (f32, f32)),
  pub line2: ((f32, f32), (f32, f32)),
  pub friend_face: (u32, u32),
  pub friend_name: (u32, u32),
  pub shakes: (u32, u32),
  pub mode: (u32, u32),
  pub status: Status,
}

#[derive(Debug, Clone)]
pub struct Status {
  pub pos: (u32, u32),
  pub max: u32,
}

#[derive(Debug, Clone)]
pub struct DisplayImpl {
  pub display: Option<RgbaImage>,
  pub name: String,
  pub layout: Layout,
}

impl Default for DisplayImpl {
  fn default() -> Self {
    Self {
      display: None,
      name: config().main.name.to_string(),
      layout: Layout {
        width: 0,
        height: 0,
        face: (0, 0),
        name: (0, 0),
        channel: (0, 0),
        aps: (0, 0),
        uptime: (0, 0),
        line1: ((0.0, 0.0), (0.0, 0.0)),
        line2: ((0.0, 0.0), (0.0, 0.0)),
        friend_face: (0, 0),
        friend_name: (0, 0),
        shakes: (0, 0),
        mode: (0, 0),
        status: Status { pos: (0, 0), max: 40 },
      },
    }
  }
}

impl Default for Layout {
  fn default() -> Self {
    Self {
      width: 0,
      height: 0,
      face: (0, 0),
      name: (0, 0),
      channel: (0, 0),
      aps: (0, 0),
      uptime: (0, 0),
      line1: ((0.0, 0.0), (0.0, 0.0)),
      line2: ((0.0, 0.0), (0.0, 0.0)),
      friend_face: (0, 0),
      friend_name: (0, 0),
      shakes: (0, 0),
      mode: (0, 0),
      status: Status { pos: (0, 0), max: 40 },
    }
  }
}

impl DisplayTrait for DisplayImpl {
  fn layout(&self) -> &Layout {
    &self.layout
  }

  fn initialize(&self) {
    println!("Not Implemented");
  }

  fn render(&self, _canvas: &mut RgbaImage) {
    println!("Not Implemented");
  }

  fn clear(&self) {
    println!("Not Implemented");
  }
}

impl DisplayImpl {
  pub fn default_fallback() -> Self {
    Self::default()
  }
}

pub fn get_display_from_config() -> Arc<dyn DisplayTrait + Send + Sync> {
  match config().ui.display.r#type.as_str() {
    "waveshare_v4" => Arc::new(Waveshare2in13bV4::new()),
    _ => Arc::new(DisplayImpl::default_fallback()),
  }
}
