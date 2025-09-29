use pwnagotchi_shared::traits::ui::Widget;
use rgb::Rgba;
use tiny_skia::{Color as SkiaColor, Paint, PathBuilder, PixmapMut as RgbaImage, Rect, Stroke};

use crate::ui::{draw::draw_text_mut, fonts::DEFAULT_FONTNAME};

#[derive(Clone)]
pub struct TextStyle {
  pub font: String,
  pub color: Rgba<u8>,
  pub size: f32,
  pub weight: cosmic_text::Weight,
  pub max_length: Option<usize>,
  pub wrap: bool,
}

impl Default for TextStyle {
  fn default() -> Self {
    Self {
      font: DEFAULT_FONTNAME.to_string(),
      color: Rgba { r: 255, g: 255, b: 255, a: 255 },
      size: 14.0,
      weight: cosmic_text::Weight::NORMAL,
      max_length: None,
      wrap: false,
    }
  }
}

pub struct Line {
  xy: ((f32, f32), (f32, f32)),
  color: Rgba<u8>,
  #[allow(dead_code)]
  width: u32,
}

impl Line {
  #[must_use]
  pub const fn new(xy: ((f32, f32), (f32, f32)), color: Rgba<u8>, width: u32) -> Self {
    Self { xy, color, width }
  }
}

impl Widget for Line {
  fn draw(&self, canvas: &mut RgbaImage) {
    let mut pb = PathBuilder::new();
    pb.move_to(self.xy.0.0, self.xy.0.1);
    pb.line_to(self.xy.1.0, self.xy.1.1);
    if let Some(path) = pb.finish() {
      let mut paint = Paint::default();
      paint.set_color(SkiaColor::from_rgba8(
        self.color.r,
        self.color.g,
        self.color.b,
        self.color.a,
      ));
      let stroke = Stroke {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        width: self.width as f32,
        ..Stroke::default()
      };
      canvas.stroke_path(&path, &paint, &stroke, tiny_skia::Transform::identity(), None);
    }
  }
  fn set_value(&mut self, _value: &str) {}
  fn get_value(&self) -> &'static str {
    ""
  }
}

pub struct RectWidget {
  rect: Rect,
  color: Rgba<u8>,
}

impl RectWidget {
  #[must_use]
  pub const fn new(rect: Rect, color: Rgba<u8>) -> Self {
    Self { rect, color }
  }
}

impl Widget for RectWidget {
  fn draw(&self, canvas: &mut RgbaImage) {
    let mut pb = PathBuilder::new();
    let x = self.rect.left();
    let y = self.rect.top();
    let w = self.rect.width();
    let h = self.rect.height();

    pb.move_to(x, y);
    pb.line_to(x + w, y);
    pb.line_to(x + w, y + h);
    pb.line_to(x, y + h);
    pb.close();

    let path = pb.finish().unwrap();
    let mut paint = Paint::default();
    paint.set_color(SkiaColor::from_rgba8(self.color.r, self.color.g, self.color.b, self.color.a));
    let stroke = Stroke { width: 1.0, ..Stroke::default() };
    canvas.stroke_path(&path, &paint, &stroke, tiny_skia::Transform::identity(), None);
  }
  fn set_value(&mut self, _value: &str) {}
  fn get_value(&self) -> &'static str {
    ""
  }
}

pub struct FilledRect {
  rect: Rect,
  color: Rgba<u8>,
}

impl FilledRect {
  #[must_use]
  pub const fn new(rect: Rect, color: Rgba<u8>) -> Self {
    Self { rect, color }
  }
}

impl Widget for FilledRect {
  fn draw(&self, canvas: &mut RgbaImage) {
    let mut pb = PathBuilder::new();
    let x = self.rect.left();
    let y = self.rect.top();
    let w = self.rect.width();
    let h = self.rect.height();

    pb.move_to(x, y);
    pb.line_to(x + w, y);
    pb.line_to(x + w, y + h);
    pb.line_to(x, y + h);
    pb.close();

    if let Some(path) = pb.finish() {
      let mut paint = Paint::default();
      paint.set_color(SkiaColor::from_rgba8(
        self.color.r,
        self.color.g,
        self.color.b,
        self.color.a,
      ));
      canvas.fill_path(
        &path,
        &paint,
        tiny_skia::FillRule::Winding,
        tiny_skia::Transform::identity(),
        None,
      );
    }
  }
  fn set_value(&mut self, _value: &str) {}
  fn get_value(&self) -> &'static str {
    ""
  }
}

pub struct TextWidget {
  xy: (u32, u32),
  value: String,
  style: TextStyle,
}

impl TextWidget {
  #[must_use]
  pub const fn new(xy: (u32, u32), value: String, style: TextStyle) -> Self {
    Self { xy, value, style }
  }
}

impl Widget for TextWidget {
  fn draw(&self, canvas: &mut RgbaImage) {
    draw_text_mut(
      &self.value,
      canvas,
      self.xy,
      &self.style.font,
      self.style.color,
      self.style.size,
      self.style.weight,
    );
  }

  fn set_value(&mut self, value: &str) {
    self.value = value.to_string();
  }

  fn get_value(&self) -> &str {
    &self.value
  }
}

pub struct LabeledValue {
  xy: (u32, u32),
  label: String,
  value: String,
  value_style: TextStyle,
  label_style: TextStyle,
}

impl LabeledValue {
  pub fn new(
    xy: (u32, u32),
    label: impl Into<String>,
    label_style: TextStyle,
    value: impl Into<String>,
    value_style: TextStyle,
  ) -> Self {
    Self {
      xy,
      label: label.into(),
      value: value.into(),
      label_style,
      value_style,
    }
  }
}

impl Widget for LabeledValue {
  fn draw(&self, canvas: &mut RgbaImage) {
    let w = draw_text_mut(
      &self.label,
      canvas,
      self.xy,
      &self.label_style.font,
      self.label_style.color,
      self.label_style.size,
      self.label_style.weight,
    );

    draw_text_mut(
      &self.value,
      canvas,
      (self.xy.0 + w + 3, self.xy.1),
      &self.value_style.font,
      self.value_style.color,
      self.value_style.size,
      self.value_style.weight,
    );
  }

  fn set_value(&mut self, value: &str) {
    self.value = value.to_string();
  }

  fn get_value(&self) -> &str {
    &self.value
  }
}
