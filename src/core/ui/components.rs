use image::{ ImageBuffer, Pixel, Rgb, RgbImage };
use imageproc::{ definitions::Image, drawing::{ draw_text, draw_text_mut, Blend, Canvas } };

pub struct Widget {
  pub xy: (i32, i32),
  pub color: i32,
}

impl Widget {
  pub fn new(x: i32, y: i32, color: i32) -> Self {
    Self {
      xy: (x, y),
      color,
    }
  }

  pub fn draw(&mut self, image: &mut RgbImage, content: &str) {
    // Load font bytes from a .ttf file (ensure the path is correct)
    /*let font = ab_glyph::FontArc::try_from_slice(include_bytes!("DejaVuSans.ttf")).unwrap();

    draw_text_mut(
      image,
      Rgb([self.color as u8, self.color as u8, self.color as u8]),
      self.xy.0,
      self.xy.1,
      20.0,
      &font,
      content,
    );*/
  }
}
