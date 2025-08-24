use std::{fs, process::Command};

use regex::Regex;



pub fn iface_channels(name: &str) -> Vec<u8> {
   let phy_out = match Command::new("/sbin/iw")
        .args(["dev", name, "info"])
        .output() {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to execute iw dev info: {e}");
                return vec![];
            }
        };

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

    let chan_out = match Command::new("/sbin/iw")
        .args([&format!("phy{phy_id}"), "channels"])
        .output() {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to execute iw phy channels: {e}");
                return vec![];
            }
        };

    if !chan_out.status.success() {
        return vec![];
    }

    let chan_str = String::from_utf8_lossy(&chan_out.stdout);

    let re = match Regex::new(r"\[(\d+)\]") {
        Ok(regex) => regex,
        Err(e) => {
            eprintln!("Failed to compile regex: {e}");
            return vec![];
        }
    };
    let mut channels = Vec::new();
    for cap in re.captures_iter(&chan_str) {
        if let Some(m) = cap.get(1)
            && let Ok(ch) = m.as_str().parse::<u8>() {
                channels.push(ch);
            }
    }
    channels
}

pub fn total_unique_handshakes(handshakes_path: &str) -> u32 {
    let mut total = 0;

    if let Ok(entries) = fs::read_dir(handshakes_path) {
        for entry in entries.filter_map(Result::ok) {
            if entry.path().extension().is_some_and(|ext| ext == "cap") {
                total += 1;
            }
        }
    }

    total
}