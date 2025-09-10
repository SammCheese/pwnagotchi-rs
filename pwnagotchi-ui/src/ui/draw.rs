#![allow(
  clippy::cast_possible_wrap,
  clippy::cast_possible_truncation,
  clippy::cast_precision_loss,
  clippy::cast_sign_loss
)]

use cosmic_text::{Attrs, Buffer, Color, Family, Metrics, Shaping, Weight};
use pwnagotchi_shared::logger::LOGGER;
use rgb::Rgba;
use tiny_skia::{BlendMode, Paint, PixmapMut as RgbaImage, Rect, Transform};

use crate::ui::fonts::{FONT_CACHE, FONTS};

/// Draws text onto a mutable RGBA image canvas at the specified position, font,
/// color, and size.
///
/// # Panics
/// This function will panic if locking the font system or font cache mutex
/// fails.
pub fn draw_text_mut(
  content: &str,
  canvas: &mut RgbaImage,
  pos: (u32, u32),
  font: &str,
  color: Rgba<u8>,
  size: f32,
  style: Weight,
) -> u32 {
  let mut font_system = match FONTS.lock() {
    Ok(fs) => fs,
    Err(poisoned) => poisoned.into_inner(),
  };

  let mut swash_cache = match FONT_CACHE.lock() {
    Ok(cache) => cache,
    Err(poisoned) => poisoned.into_inner(),
  };

  let requested_exists = font_system
    .db()
    .faces()
    .any(|f| f.families.iter().any(|(name, _)| name == font));

  let resolved_family: String = if requested_exists {
    font.to_string()
  } else {
    let fallback = font_system
      .db()
      .faces()
      .next()
      .and_then(|f| f.families.first().map(|(name, _)| name.clone()))
      .unwrap_or_else(|| "DejaVu Sans Mono".to_string());

    LOGGER.log_warning("Fonts", &format!("Font '{font}' not found, using '{fallback}'."));

    fallback
  };

  let fontsize = size;
  let lineheight = size * 1.2;
  let metrics = Metrics::new(fontsize, lineheight);
  let mut buffer = Buffer::new(&mut font_system, metrics);
  let mut buffer = buffer.borrow_with(&mut font_system);
  let canvas_width = canvas.width();
  let canvas_height = canvas.height();
  let avail_w = canvas_width.saturating_sub(pos.0) as f32;
  let avail_h = canvas_height.saturating_sub(pos.1) as f32;

  buffer.set_size(Some(avail_w), Some(avail_h));

  let mut attrs = Attrs::new().family(Family::Name(&resolved_family));
  attrs = attrs.weight(style);

  buffer.set_text(content, &attrs, Shaping::Advanced);
  buffer.shape_until_scroll(true);

  let base_color = Color::rgba(color.r, color.g, color.b, color.a);
  let off_x = pos.0 as i32;
  let off_y = pos.1 as i32;

  let mut min_draw_x: i32 = i32::MAX;
  let mut max_draw_x: i32 = i32::MIN;

  buffer.draw(&mut swash_cache, base_color, |x, y, w, h, c| {
    let a = c.a();
    if a == 0 {
      return;
    }

    if w > 0 {
      min_draw_x = min_draw_x.min(x);
      max_draw_x = max_draw_x.max(x + (w as i32));
    }

    let mut paint = Paint {
      anti_alias: false,
      blend_mode: BlendMode::SourceOver,
      ..Default::default()
    };
    paint.set_color_rgba8(c.r(), c.g(), c.b(), a);

    let rect = Rect::from_xywh((x + off_x) as f32, (y + off_y) as f32, w as f32, h as f32).unwrap();

    canvas.fill_rect(rect, &paint, Transform::identity(), None);
  });
  drop(font_system);

  let width_px: u32 = if min_draw_x <= max_draw_x { (max_draw_x - min_draw_x) as u32 } else { 0 };

  width_px
}
