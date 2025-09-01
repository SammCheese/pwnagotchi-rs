use image::{ImageFormat, RgbaImage};
use std::fs;
use std::path::Path;

pub const FRAME_FOLDER: &str = "/var/tmp/pwnagotchi";
pub static FRAME_PATH: std::sync::LazyLock<String> =
  std::sync::LazyLock::new(|| format!("{FRAME_FOLDER}/pwnagotchi.png"));
pub const FORMAT: ImageFormat = ImageFormat::Png;
pub const CTYPE: &str = "image/png";

pub static FRAME_LOCK: std::sync::LazyLock<std::sync::Mutex<()>> =
  std::sync::LazyLock::new(|| std::sync::Mutex::new(()));

#[allow(clippy::missing_errors_doc)]
pub fn update_frame(img: &RgbaImage) -> image::ImageResult<()> {
  tokio::task::block_in_place(|| {
    let _guard = FRAME_LOCK
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner);

    if !Path::new(FRAME_FOLDER).exists() {
      fs::create_dir_all(FRAME_FOLDER)?;
    }

    let tmp_path = format!("{}.tmp", FRAME_PATH.as_str());
    img.save_with_format(&tmp_path, FORMAT)?;

    fs::rename(&tmp_path, FRAME_PATH.as_str())?;

    Ok(())
  })
}
