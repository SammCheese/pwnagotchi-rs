use std::{ path::Path, sync::Arc };

use ab_glyph::{ Font, FontArc, PxScale, ScaleFont };
use image::{ DynamicImage, Rgba, RgbaImage };
use imageproc::{
  drawing::{ draw_filled_rect_mut, draw_hollow_rect_mut, draw_line_segment_mut, draw_text_mut },
  rect::Rect,
};
use crate::core::{ log::LOGGER, ui::state::StateValue };

pub trait Widget: Send + Sync {
  fn draw(&self, canvas: &mut RgbaImage);
  fn set_value(&mut self, value: &StateValue);
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
  /// Returns an `image::ImageError` if the image cannot be opened from the provided path.
  pub fn new<P: AsRef<Path>>(
    path: P,
    xy: (u32, u32),
    color: u8
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
  fn set_value(&mut self, _value: &StateValue) {
  }
  fn get_value(&self) -> StateValue {
    StateValue::None
  }
}

// === Line ===
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
  fn set_value(&mut self, _value: &StateValue) {
  }
  fn get_value(&self) -> StateValue {
    StateValue::None
  }
}

// === Rect ===
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
  fn set_value(&mut self, _value: &StateValue) {
  }
  fn get_value(&self) -> StateValue {
    StateValue::None
  }
}

// === FilledRect ===
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
  fn set_value(&mut self, _value: &StateValue) {
  }
  fn get_value(&self) -> StateValue {
    StateValue::None
  }
}

// === Text ===
pub struct TextWidget {
  xy: (u32, u32),
  value: String,
  font: Arc<FontArc>,
  color: Rgba<u8>,
  size: f32,
}

impl TextWidget {
  pub fn new(
    xy: (u32, u32),
    value: impl Into<String>,
    font: Arc<FontArc>,
    color: Rgba<u8>,
    size: f32
  ) -> Self {
    Self { xy, value: value.into(), font, color, size }
  }
}

impl Widget for TextWidget {
  fn draw(&self, canvas: &mut RgbaImage) {
    let scale = PxScale::from(self.size);
    draw_text_mut(
      canvas,
      self.color,
      self.xy.0.try_into().unwrap_or_else(|_| {
        LOGGER.log_error("UI", "Failed to convert x coordinate in TextWidget::draw");
        0
      }),
      self.xy.1.try_into().unwrap_or_else(|_| {
        LOGGER.log_error("UI", "Failed to convert y coordinate in TextWidget::draw");
        0
      }),
      scale,
      &*self.font,
      &self.value
    );
  }
  fn set_value(&mut self, value: &StateValue) {
    match value {
      StateValue::None => {}
      StateValue::Face(face) => {
        self.value = format!("{face:?}");
      }
      StateValue::Text(text) => {
        self.value.clone_from(text);
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

pub struct LabeledValue {
  xy: (u32, u32),
  label: String,
  value: String,
  font: Arc<FontArc>,
  color: Rgba<u8>,
  size: f32,
}

impl LabeledValue {
  pub fn new(
    xy: (u32, u32),
    label: impl Into<String>,
    value: impl Into<String>,
    font: Arc<FontArc>,
    color: Rgba<u8>,
    size: f32
  ) -> Self {
    Self { xy, label: label.into(), value: value.into(), font, color, size }
  }
}

impl Widget for LabeledValue {
  fn draw(&self, canvas: &mut RgbaImage) {
    let scale = PxScale::from(self.size);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let label_width = self.label
      .chars()
      .map(|c| {
        let gid = self.font.glyph_id(c);
        let scale = PxScale::from(self.size);
        self.font.as_scaled(scale).h_advance(gid)
      })
      .sum::<f32>() as u32;

    draw_text_mut(
      canvas,
      self.color,
      self.xy.0.try_into().unwrap_or_else(|_| {
        LOGGER.log_error("UI", "Failed to convert x coordinate in LabeledValue::draw");
        0
      }),
      self.xy.1.try_into().unwrap_or_else(|_| {
        LOGGER.log_error("UI", "Failed to convert y coordinate in LabeledValue::draw");
        0
      }),
      scale,
      &*self.font,
      &self.label
    );

    draw_text_mut(
      canvas,
      self.color,
      self.xy.0
        .checked_add(label_width + 5)
        .unwrap_or(self.xy.0)
        .try_into()
        .unwrap_or_else(|_| {
          LOGGER.log_error(
            "UI",
            "Failed to convert x coordinate in LabeledValue::draw (value part)"
          );
          0
        }),
      self.xy.1.try_into().unwrap_or_else(|_| {
        LOGGER.log_error("UI", "Failed to convert y coordinate in LabeledValue::draw (value part)");
        0
      }),
      scale,
      &*self.font,
      &self.value
    );
  }
  fn set_value(&mut self, value: &StateValue) {
    match value {
      StateValue::None => {}
      StateValue::Face(face) => {
        self.value = format!("{face:?}");
      }
      StateValue::Text(text) => {
        self.value.clone_from(text);
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
