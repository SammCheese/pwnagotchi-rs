#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
use cosmic_text::{Weight, Attrs, Buffer, CacheKeyFlags, Color, Family, Metrics, Shaping};
use image::{Rgba, RgbaImage};

use crate::core::{
    log::LOGGER,
    ui::fonts::{FONT_CACHE, FONTS},
};

/// Draws text onto a mutable RGBA image canvas at the specified position, font, color, and size.
///
/// # Panics
/// This function will panic if locking the font system or font cache mutex fails.
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
            .unwrap_or_else(|| "Sans-Serif".to_string());
        LOGGER.log_warning(
            "Fonts",
            &format!("Font '{font}' not found, using '{fallback}'."),
        );
        fallback
    };

    let fontsize = size;
    let lineheight = size * 1.2;
    let metrics = Metrics::new(fontsize, lineheight);

    let mut buffer = Buffer::new(&mut font_system, metrics);
    let mut buffer = buffer.borrow_with(&mut font_system);

    let canvas_width = canvas.width();
    let canvas_height = canvas.height();
    let canvas_width_i32 = canvas_width as i32;
    let canvas_height_i32 = canvas_height as i32;

    let avail_w = canvas_width.saturating_sub(pos.0) as f32;
    let avail_h = canvas_height.saturating_sub(pos.1) as f32;
    buffer.set_size(Some(avail_w), Some(avail_h));

    let mut attrs = Attrs::new().family(Family::Name(&resolved_family));
    attrs = attrs.cache_key_flags(CacheKeyFlags::DISABLE_HINTING);
    attrs = attrs.weight(style);
    buffer.set_text(content, &attrs, Shaping::Advanced);
    buffer.shape_until_scroll(true);

    let base_color = Color::rgba(color[0], color[1], color[2], color[3]);

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
            max_draw_x = max_draw_x.max(x + w as i32);
        }

        for dy in 0..h {
            for dx in 0..w {
                let tx = x + off_x + (dx as i32);
                let ty = y + off_y + (dy as i32);

                if tx < 0 || ty < 0 || tx >= canvas_width_i32 || ty >= canvas_height_i32 {
                    continue;
                }

                let dst = canvas.get_pixel(tx as u32, ty as u32).0;
                let src_a = u32::from(a);
                let inv_a = 255u32 - src_a;

                let blend = |s: u8, d: u8| -> u8 {
                    ((u32::from(s) * src_a + u32::from(d) * inv_a) / 255).min(255) as u8
                };

                let r = blend(c.r(), dst[0]);
                let g = blend(c.g(), dst[1]);
                let b = blend(c.b(), dst[2]);
                let out_a = (src_a + (u32::from(dst[3]) * inv_a) / 255).min(255) as u8;

                canvas.put_pixel(tx as u32, ty as u32, Rgba([r, g, b, out_a]));
            }
        }
    });

    let width_px: u32 = if min_draw_x <= max_draw_x {
        (max_draw_x - min_draw_x) as u32
    } else {
        0
    };
    width_px
}
