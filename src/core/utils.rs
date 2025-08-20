use std::process::Command;

use regex::Regex;



pub fn iface_channels(name: &str) -> Vec<u8> {
   let phy_out = Command::new("/sbin/iw")
        .args(["dev", name, "info"])
        .output()
        .expect("Failed to execute iw dev info");

    if !phy_out.status.success() {
        return vec![];
    }

    let phy_str = String::from_utf8_lossy(&phy_out.stdout);
    let phy_id = phy_str
        .lines()
        .find_map(|line| {
            if line.trim_start().starts_with("wiphy") {
                line.split_whitespace().nth(1)
            } else {
                None
            }
        })
        .unwrap_or("");

    if phy_id.is_empty() {
        return vec![];
    }

    let chan_out = Command::new("/sbin/iw")
        .args([&format!("phy{}", phy_id), "channels"])
        .output()
        .expect("Failed to execute iw phy channels");

    if !chan_out.status.success() {
        return vec![];
    }

    let chan_str = String::from_utf8_lossy(&chan_out.stdout);

    let re = Regex::new(r"\[(\d+)\]").unwrap();
    let mut channels = Vec::new();
    for cap in re.captures_iter(&chan_str) {
        if let Some(m) = cap.get(1) {
            if let Ok(ch) = m.as_str().parse::<u8>() {
                channels.push(ch);
            }
        }
    }
    channels
}