use crate::core::config::Config;

const WHITE: i32 = 0x00;
const BLACK: i32 = 0xFF;

pub struct View {
  pub inverted: bool,
  pub background_color: i32,
  pub foreground_color: i32,
}

impl Default for View {
  fn default() -> Self {
    let inverted = false;
    let background_color = WHITE;
    let foreground_color = BLACK;
    Self {
      inverted,
      background_color,
      foreground_color,
    }
  }
}

impl View {
  pub const fn new(config: &Config) -> Self {
    let inverted = config.ui.inverted;
    let background_color = if inverted { BLACK } else { WHITE };
    let foreground_color = if inverted { WHITE } else { BLACK };
    Self {
      inverted,
      background_color,
      foreground_color,
    }
  }

  pub fn wait(&self, mut secs: f64, sleeping: Option<bool>) {
    let sleeping = sleeping.unwrap_or(true);
    let was_normal = true; // TODO: Change logic later
    let part = secs / 10.0;
    for step in 0..10 {
      if was_normal || step > 5 {
        if sleeping {
          if secs > 1.0 {
            // TODO
          }
        } else {
          // TODO
        }
      }
      std::thread::sleep(std::time::Duration::from_secs_f64(part));
      secs -= part;
    }
  }
}