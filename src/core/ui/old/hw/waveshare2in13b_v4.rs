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
    Self {
      base: DisplayImpl::default(),
    }
  }
}

impl DisplayTrait for Waveshare2in13bV4 {
  fn layout(&mut self) -> &Layout {
    let layout = &mut self.base.layout;
    layout.width = 250;
    layout.height = 122;
    layout.face = (0, 40);
    layout.name = (5, 20);
    layout.channel = (0, 0);
    layout.aps = (28, 0);
    layout.uptime = (185, 0);
    layout.line1 = ((0.0, 14.0), (250.0, 14.0));
    layout.line2 = ((0.0, 108.0), (250.0, 108.01));
    layout.friend_face = (0, 92);
    layout.friend_name = (40, 94);
    layout.shakes = (0, 109);
    layout.mode = (225, 109);
    layout.status.pos = (38, 93);

    layout
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
