use std::{fs, path::Path};

use image::{ImageFormat, RgbaImage};

pub const FRAME_FOLDER: &str = "/var/tmp/pwnagotchi";

pub static FRAME_PATH: std::sync::LazyLock<String> =
  std::sync::LazyLock::new(|| format!("{FRAME_FOLDER}/pwnagotchi.png"));

pub static FRAME_LOCK: std::sync::LazyLock<tokio::sync::Mutex<()>> =
  std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

pub const FORMAT: ImageFormat = ImageFormat::Png;
pub const CTYPE: &str = "image/png";

/// Updates the frame image on disk.
///
/// # Errors
/// Returns an error if the image cannot be saved, the directory cannot be
/// created, or the file cannot be renamed.
pub fn update_frame(img: &RgbaImage) -> image::ImageResult<()> {
  let _guard = FRAME_LOCK.lock();

  if !Path::new(FRAME_FOLDER).exists() {
    fs::create_dir_all(FRAME_FOLDER)?;
  }

  let tmp_path = format!("{}.tmp", FRAME_PATH.as_str());

  img.save_with_format(&tmp_path, FORMAT)?;

  fs::rename(&tmp_path, FRAME_PATH.as_str())?;

  Ok(())
}
