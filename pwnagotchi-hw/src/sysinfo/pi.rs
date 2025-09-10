use std::error::Error;

use crate::sysinfo::SysInfo;

pub struct PiSysInfo;

impl SysInfo for PiSysInfo {
  fn get_temperature(&self, celsius: Option<bool>) -> Result<f32, Box<dyn Error>> {
    let celsius = celsius.unwrap_or(true);
    let temp = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
      .map_err(|e| Box::new(e) as Box<dyn Error>)
      .and_then(|s| {
        s.trim()
          .parse::<f32>()
          .map_err(|e| Box::new(e) as Box<dyn Error>)
          .map(|t| t / 1000.0)
      });

    match (temp, celsius) {
      (Ok(t), true) => Ok(t),
      (Ok(t), false) => Ok(t * 9.0 / 5.0 + 32.0),
      (Err(e), _) => Err(e),
    }
  }

  fn get_uptime(&self) -> Result<u64, Box<dyn Error>> {
    let uptime_str =
      std::fs::read_to_string("/proc/uptime").map_err(|e| Box::new(e) as Box<dyn Error>)?;
    Ok(uptime_str.split(".").next().unwrap_or("0").parse::<u64>().unwrap_or(0))
  }

  fn get_cpu_usage(&self) -> Result<f32, Box<dyn Error>> {
    Ok(0.0)
  }

  fn get_memory_usage(&self) -> Result<f32, Box<dyn Error>> {
    std::fs::read("/proc/meminfo")
      .map_err(|e| Box::new(e) as Box<dyn Error>)
      .map(|data| {
        let meminfo = String::from_utf8_lossy(&data);
        let mut total = 0.0;
        let mut free = 0.0;
        let mut buff: f32 = 0.0;
        let mut cached: f32 = 0.0;
        for line in meminfo.lines() {
          if line.starts_with("MemTotal:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
              total = parts[1].parse::<f32>().unwrap_or(0.0);
            }
          } else if line.starts_with("MemFree:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
              free = parts[1].parse::<f32>().unwrap_or(0.0);
            }
          } else if line.starts_with("Buffers:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
              buff = parts[1].parse::<f32>().unwrap_or(0.0)
            }
          } else if line.starts_with("Cached:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
              cached = parts[1].parse::<f32>().unwrap_or(0.0);
            }
          }
        }
        let used = total - free - buff - cached;
        (used / total).round()
      })
  }
}
