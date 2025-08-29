use ab_glyph::{point, Font, GlyphId, OutlinedGlyph, PxScale, Rect, ScaleFont};
use image::{GenericImage, Pixel};
use imageproc::definitions::{Clamp, Image};
use imageproc::drawing::Canvas;

/// Layout glyphs and call back with their outlines and bounding boxes.
fn layout_glyphs(
    scale: impl Into<PxScale> + Copy,
    font: &impl Font,
    text: &str,
    mut f: impl FnMut(OutlinedGlyph, Rect),
) -> (u32, u32) {
    if text.is_empty() {
        return (0, 0);
    }
    let font = font.as_scaled(scale);

    let mut w = 0.0;
    let mut prev: Option<GlyphId> = None;

    for c in text.chars() {
        let glyph_id = font.glyph_id(c);
        let glyph = glyph_id.with_scale_and_position(scale, point(w, font.ascent()));
        w += font.h_advance(glyph_id);
        if let Some(g) = font.outline_glyph(glyph) {
            if let Some(prev) = prev {
                w += font.kern(glyph_id, prev);
            }
            prev = Some(glyph_id);
            let bb = g.px_bounds();
            f(g, bb);
        }
    }

    let w = w.ceil();
    let h = font.height().ceil();
    assert!(w >= 0.0);
    assert!(h >= 0.0);
    (1 + w as u32, h as u32)
}

/// Get the width and height of the given text.
pub fn text_size(scale: impl Into<PxScale> + Copy, font: &impl Font, text: &str) -> (u32, u32) {
    layout_glyphs(scale, font, text, |_, _| {})
}

/// Draw text, returning a new image.
pub fn draw_text<I>(
    image: &I,
    color: I::Pixel,
    x: i32,
    y: i32,
    scale: impl Into<PxScale> + Copy,
    font: &impl Font,
    text: &str,
) -> Image<I::Pixel>
where
    I: GenericImage,
    <I::Pixel as Pixel>::Subpixel: Into<f32> + Clamp<f32>,
{
    let mut out = Image::new(image.width(), image.height());
    out.copy_from(image, 0, 0).unwrap();
    draw_text_mut(&mut out, color, x, y, scale, font, text);
    out
}

/// Draws non-antialiased text directly onto a canvas.
pub fn draw_text_mut<C>(
    canvas: &mut C,
    color: C::Pixel,
    x: i32,
    y: i32,
    scale: impl Into<PxScale> + Copy,
    font: &impl Font,
    text: &str,
) where
    C: Canvas,
    <C::Pixel as Pixel>::Subpixel: Into<f32> + Clamp<f32>,
{
    let image_width = canvas.width() as i32;
    let image_height = canvas.height() as i32;

    layout_glyphs(scale, font, text, |g, bb| {
        let x_shift = x + bb.min.x.round() as i32;
        let y_shift = y + bb.min.y.round() as i32;
        g.draw(|gx, gy, gv| {
            let image_x = gx as i32 + x_shift;
            let image_y = gy as i32 + y_shift;

            if (0..image_width).contains(&image_x) && (0..image_height).contains(&image_y) {
                // Disable AA: snap gv to 0 or 1
                if gv >= 0.35 {
                    canvas.draw_pixel(image_x as u32, image_y as u32, color);
                }
            }
        });
    });
}
