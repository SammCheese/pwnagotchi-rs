use std::{hash::Hash, sync::Mutex};

use cosmic_text::{FontSystem, SwashCache};

use crate::web::server::FONT_ASSETS;

pub const DEFAULT_FONTNAME: &str = "DejaVu Sans Mono";
pub static STATUS_FONT_NAME: &str = DEFAULT_FONTNAME;
pub static SIZE_OFFSET: f32 = 0.0;

#[derive(Eq, Hash, PartialEq)]
pub enum FontType {
  Regular,
  Bold,
  BoldSmall,
  BoldBig,
  Medium,
  Small,
  Huge,
}

fn get_default_font() -> cosmic_text::fontdb::Source {
  cosmic_text::fontdb::Source::Binary(std::sync::Arc::new(
    FONT_ASSETS.get_file("DejaVuSans.ttf").unwrap().contents().to_vec(),
  ))
}

pub static FONTS: std::sync::LazyLock<Mutex<FontSystem>> =
  std::sync::LazyLock::new(|| Mutex::new(FontSystem::new_with_fonts(vec![get_default_font()])));

pub static FONT_CACHE: std::sync::LazyLock<Mutex<SwashCache>> =
  std::sync::LazyLock::new(|| Mutex::new(SwashCache::new()));
