use image::RgbaImage;

use crate::core::ui::old::hw::base::{DisplayImpl, DisplayTrait, Layout};

#[derive(Debug, Clone)]

pub struct Waveshare2in13bV4 {
  pub base: DisplayImpl,
}

impl Default for Waveshare2in13bV4 {
  fn default() -> Self {
    Self::new()
  }
}

impl Waveshare2in13bV4 {
  pub fn new() -> Self {
    let mut base = DisplayImpl::default();
    base.layout.width = 250;
    base.layout.height = 122;
    base.layout.face = (0, 40);
    base.layout.name = (5, 20);
    base.layout.channel = (0, 0);
    base.layout.aps = (28, 0);
    base.layout.uptime = (185, 0);
    base.layout.line1 = ((0.0, 14.0), (250.0, 14.0));
    base.layout.line2 = ((0.0, 108.0), (250.0, 108.01));
    base.layout.friend_face = (0, 92);
    base.layout.friend_name = (40, 94);
    base.layout.shakes = (0, 109);
    base.layout.mode = (225, 109);
    base.layout.status.pos = (38, 93);
    Self { base }
  }
}

impl DisplayTrait for Waveshare2in13bV4 {
  fn layout(&self) -> &Layout {
    &self.base.layout
  }

  fn initialize(&self) {
    // Initialization logic here
  }

  fn render(&self, _canvas: &mut RgbaImage) {
    // Rendering logic here
  }

  fn clear(&self) {
    // Clearing logic here
  }
}
