use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct FSConfig {
  pub memory: FSMemoryConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct FSMemoryConfig {
  pub enabled: bool,
  pub mounts: MountsConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
#[derive(Default)]
pub struct MountsConfig {
  pub log: MountConfig,
  pub data: MountConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct MountConfig {
  pub enabled: bool,
  pub mount: String,
  pub size: String,
  pub sync: u64,
  pub zram: bool,
  pub rsync: bool,
}

impl Default for MountConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      mount: String::from("/etc/pwnagotchi/log/"),
      size: String::from("10M"),
      sync: 3600,
      zram: false,
      rsync: false,
    }
  }
}

impl Default for FSMemoryConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      mounts: MountsConfig::default(),
    }
  }
}

impl Default for FSConfig {
  fn default() -> Self {
    Self {
      memory: FSMemoryConfig {
        enabled: true,
        mounts: MountsConfig {
          log: MountConfig {
            enabled: true,
            mount: String::from("/etc/pwnagotchi/log/"),
            size: String::from("50M"),
            sync: 60,
            zram: true,
            rsync: true,
          },
          data: MountConfig {
            enabled: true,
            mount: String::from("/var/tmp/pwnagotchi"),
            size: String::from("10M"),
            sync: 3600,
            zram: true,
            rsync: true,
          },
        },
      },
    }
  }
}
