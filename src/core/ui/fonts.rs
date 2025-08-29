use std::hash::Hash;
use std::{collections::HashMap, sync::Arc};
use std::sync::{Mutex};

use ab_glyph::{ FontArc, FontRef };
use fontdb::{ self, Database };


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
  pub fonts: HashMap<FontType, Arc<FontArc>>,
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
      &&
        let Some(face) = self.db
          .query(
            &(fontdb::Query {
              families: &[fontdb::Family::Name(family)],
              ..Default::default()
            })
          )
          .and_then(|id| self.db.face(id))
        && let fontdb::Source::File(path) = &face.source
          && let Ok(bytes) = std::fs::read(path) {
            self.loaded.insert(family.to_string(), bytes);
          }
    self.loaded.get(family).map(std::vec::Vec::as_slice)
  }

  pub fn get_font_ref(&mut self, family: &str) -> Option<FontRef<'_>> {
    self.get_font_bytes(family).and_then(|bytes| FontRef::try_from_slice(bytes).ok())
  }

  pub fn get_font_arc(&mut self, family: &str) -> Option<FontArc> {
    self.get_font_bytes(family).map_or_else(|| {
      eprintln!("Font bytes not found for family '{family}'");
      None
    }, |bytes| match FontArc::try_from_vec(bytes.to_vec()) {
      Ok(font_arc) => Some(font_arc),
      Err(e) => {
        eprintln!("Failed to load font '{family}': {e}");
        None
      }
    })
  }

  pub fn get_status_font(&mut self) -> Option<FontArc> {
    self.get_font_arc(STATUS_FONT_NAME)
  }
}

pub static FONTS: std::sync::LazyLock<Mutex<Fonts>> = std::sync::LazyLock::new(|| {
    let mut coll = Fonts::new();

    if let Some(regular) = coll.get_font_arc(FONTNAME) {
        coll.fonts.insert(FontType::Regular, Arc::new(regular));
    }
    Mutex::new(coll)
});
