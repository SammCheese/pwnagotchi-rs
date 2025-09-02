use std::{
  collections::HashMap,
  hash::Hash,
  sync::{Arc, Mutex},
};

use cosmic_text::{FontSystem, SwashCache};
use fontdb::{self, Database};

const FONTNAME: &str = "DejaVu Sans Mono";

pub static STATUS_FONT_NAME: &str = FONTNAME;

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

pub struct Fonts {
  db: Database,
  loaded: HashMap<String, Vec<u8>>,
  pub fonts: HashMap<FontType, Arc<Vec<u8>>>,
}

impl Default for Fonts {
  fn default() -> Self {
    Self::new()
  }
}

impl Fonts {
  pub fn new() -> Self {
    let mut db = Database::new();
    db.load_system_fonts();

    // Arch Linux for testing.
    let dir = std::path::Path::new("/usr/share/fonts/TTF/");

    db.load_fonts_dir(dir);
    Self {
      db,
      loaded: HashMap::new(),
      fonts: HashMap::new(),
    }
  }

  pub fn get_font_bytes(&mut self, family: &str) -> Option<&[u8]> {
    if !self.loaded.contains_key(family)
      && let Some(face) = self
        .db
        .query(
          &(fontdb::Query {
            families: &[fontdb::Family::Name(family)],
            ..Default::default()
          }),
        )
        .and_then(|id| self.db.face(id))
      && let fontdb::Source::File(path) = &face.source
      && let Ok(bytes) = std::fs::read(path)
    {
      self.loaded.insert(family.to_string(), bytes);
    }

    self.loaded.get(family).map(std::vec::Vec::as_slice)
  }

  pub fn get_status_font_bytes(&mut self) -> Option<Arc<Vec<u8>>> {
    self.get_font_bytes(STATUS_FONT_NAME).map(|b| Arc::new(b.to_vec()))
  }
}

pub static FONTS: std::sync::LazyLock<Mutex<FontSystem>> =
  std::sync::LazyLock::new(|| Mutex::new(FontSystem::new()));

pub static FONT_CACHE: std::sync::LazyLock<Mutex<SwashCache>> =
  std::sync::LazyLock::new(|| Mutex::new(SwashCache::new()));
