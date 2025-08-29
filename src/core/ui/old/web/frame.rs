use image::{ImageFormat, RgbaImage};
use std::path::Path;
use std::fs;

//pub const FRAME_PATH: &str = "/var/tmp/pwnagotchi/pwnagotchi.png";
pub const FRAME_PATH: &str = "/home/sammy/Public/pwnagotchi-rs/test/pwnagotchi.png";
pub const FRAME_FOLDER: &str = "/home/sammy/Public/pwnagotchi-rs/test";
pub const FORMAT: ImageFormat = ImageFormat::Png;
pub const CTYPE: &str = "image/png";

// Use a synchronous lock here because this function is synchronous; this avoids partial writes.
pub static FRAME_LOCK: std::sync::LazyLock<std::sync::Mutex<()>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(()));

pub fn update_frame(img: &RgbaImage) -> image::ImageResult<()>  {
  // Take the synchronous lock; proceed even if the lock has been poisoned
  let _guard = FRAME_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

  if !Path::new(FRAME_FOLDER).exists() {
    fs::create_dir_all(FRAME_FOLDER)?;
  }

  // Write atomically: save to a temporary file then rename over the target.
  let tmp_path = format!("{}.tmp", FRAME_PATH);
  img.save_with_format(&tmp_path, FORMAT)?;
  // Best-effort atomic replace on the same filesystem
  fs::rename(&tmp_path, FRAME_PATH)?;

  Ok(())
}