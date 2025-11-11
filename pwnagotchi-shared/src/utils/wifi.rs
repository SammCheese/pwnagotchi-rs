#![allow(clippy::cast_possible_truncation)]

pub static NUM_CHANNELS: i16 = 233;

// Frequency in MHz to channel number
pub fn freq_to_channel(freq: f32) -> i16 {
  match freq {
    // 2.4 Ghz
    2412.0..=2472.0 => ((freq - 2412.0) / 5.0).round() as i16,
    // Channel 14 is special
    2484.0 => 14,
    // 5GHz
    5150.0..=5850.0 => {
      // Channel 36-64
      if (5150.0..=5350.0).contains(&freq) {
        (((freq - 5150.0) / 20.0).round() as i16) + 36
        // Channels 100-144
      } else if (5470.0..=5725.0).contains(&freq) {
        (((freq - 5500.0) / 20.0).round() as i16) + 100
        // Channels 149-165
      } else {
        (((freq - 5745.0) / 20.0).round() as i16) + 149
      }
    }
    // 6GHz
    5925.0..=7125.0 => (((freq - 5925.0) / 20.0).round() as i16) + 11,
    _ => -1,
  }
}
