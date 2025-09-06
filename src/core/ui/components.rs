use std::path::Path;

use image::{DynamicImage, Rgba, RgbaImage};
use imageproc::{
  drawing::{draw_filled_rect_mut, draw_hollow_rect_mut, draw_line_segment_mut},
  rect::Rect,
};

use crate::core::ui::{draw::draw_text_mut, state::StateValue};

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
      font: "DejaVu Sans Mono".to_string(),
      color: Rgba([0, 0, 0, 255]),
      size: 14.0,
      weight: cosmic_text::Weight::NORMAL,
      max_length: None,
      wrap: false,
    }
  }
}

pub trait Widget: Send + Sync {
  fn draw(&self, canvas: &mut RgbaImage);
  fn set_value(&mut self, value: StateValue);
  fn get_value(&self) -> StateValue;
}

pub struct Bitmap {
  xy: (u32, u32),
  color: u8,
  image: DynamicImage,
}

impl Bitmap {
  /// Creates a new `Bitmap` from the given path, position, and color.
  ///
  /// # Errors
  ///
  /// Returns an error if the image cannot be opened from the provided path.
  pub fn new<P: AsRef<Path>>(
    path: P,
    xy: (u32, u32),
    color: u8,
  ) -> Result<Self, image::ImageError> {
    let image = image::open(path)?;
    Ok(Self { xy, color, image })
  }
}

impl Widget for Bitmap {
  fn draw(&self, canvas: &mut RgbaImage) {
    let mut img = self.image.clone();
    if self.color == 0xff {
      img.invert();
    }

    let img_rgba = img.to_rgba8();

    image::imageops::overlay(canvas, &img_rgba, self.xy.0.into(), self.xy.1.into());
  }
  fn set_value(&mut self, _value: StateValue) {}
  fn get_value(&self) -> StateValue {
    StateValue::None
  }
}

pub struct Line {
  xy: ((f32, f32), (f32, f32)),
  color: Rgba<u8>,
  width: u32,
}

impl Line {
  pub const fn new(xy: ((f32, f32), (f32, f32)), color: Rgba<u8>, width: u32) -> Self {
    Self { xy, color, width }
  }
}

impl Widget for Line {
  fn draw(&self, canvas: &mut RgbaImage) {
    draw_line_segment_mut(canvas, self.xy.0, self.xy.1, self.color);
  }
  fn set_value(&mut self, _value: StateValue) {}
  fn get_value(&self) -> StateValue {
    StateValue::None
  }
}

pub struct RectWidget {
  rect: Rect,
  color: Rgba<u8>,
}

impl RectWidget {
  pub const fn new(rect: Rect, color: Rgba<u8>) -> Self {
    Self { rect, color }
  }
}

impl Widget for RectWidget {
  fn draw(&self, canvas: &mut RgbaImage) {
    draw_hollow_rect_mut(canvas, self.rect, self.color);
  }
  fn set_value(&mut self, _value: StateValue) {}
  fn get_value(&self) -> StateValue {
    StateValue::None
  }
}

pub struct FilledRect {
  rect: Rect,
  color: Rgba<u8>,
}

impl FilledRect {
  pub const fn new(rect: Rect, color: Rgba<u8>) -> Self {
    Self { rect, color }
  }
}

impl Widget for FilledRect {
  fn draw(&self, canvas: &mut RgbaImage) {
    draw_filled_rect_mut(canvas, self.rect, self.color);
  }
  fn set_value(&mut self, _value: StateValue) {}
  fn get_value(&self) -> StateValue {
    StateValue::None
  }
}

pub struct TextWidget {
  xy: (u32, u32),
  value: String,
  style: TextStyle,
}

impl TextWidget {
  pub fn new(xy: (u32, u32), value: impl Into<String>, style: TextStyle) -> Self {
    Self { xy, value: value.into(), style }
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

  fn set_value(&mut self, value: StateValue) {
    match value {
      StateValue::None => {}
      StateValue::Face(face) => {
        self.value = format!("{face:?}");
      }
      StateValue::Text(text) => {
        self.value = text;
      }
      StateValue::Number(num) => {
        self.value = num.to_string();
      }
      StateValue::Bool(b) => {
        self.value = b.to_string();
      }
    }
  }

  fn get_value(&self) -> StateValue {
    StateValue::Text(self.value.clone())
  }
}

struct LabelContent {
  text: String,
  style: String,
}

pub struct LabeledValue {
  xy: (u32, u32),
  label: String,
  value: String,
  style: TextStyle,
}

impl LabeledValue {
  pub fn new(
    xy: (u32, u32),
    label: impl Into<String>,
    value: impl Into<String>,
    style: TextStyle,
  ) -> Self {
    Self {
      xy,
      label: label.into(),
      value: value.into(),
      style,
    }
  }
}

impl Widget for LabeledValue {
  fn draw(&self, canvas: &mut RgbaImage) {
    let mut label_style = self.style.clone();
    label_style.weight = cosmic_text::Weight::EXTRA_BOLD;
    let w = draw_text_mut(
      &self.label,
      canvas,
      self.xy,
      &label_style.font,
      label_style.color,
      label_style.size,
      label_style.weight,
    );

    let mut value_style = self.style.clone();
    value_style.weight = cosmic_text::Weight::NORMAL;
    draw_text_mut(
      &self.value,
      canvas,
      (self.xy.0 + w + 3, self.xy.1),
      &value_style.font,
      value_style.color,
      value_style.size,
      value_style.weight,
    );
  }

  fn set_value(&mut self, value: StateValue) {
    match value {
      StateValue::None => {}
      StateValue::Face(face) => {
        self.value = format!("{face:?}");
      }
      StateValue::Text(text) => {
        self.value = text;
      }
      StateValue::Number(num) => {
        self.value = num.to_string();
      }
      StateValue::Bool(b) => {
        self.value = b.to_string();
      }
    }
  }

  fn get_value(&self) -> StateValue {
    StateValue::Text(self.value.clone())
  }
}
