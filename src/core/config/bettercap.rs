use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct BettercapConfig {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub silence: Vec<String>,
    pub handshakes: String,
}

impl Default for BettercapConfig {
    fn default() -> Self {
        Self {
            hostname: "localhost".into(),
            port: 8081,
            username: "user".into(),
            password: "pass".into(),
            silence: Vec::new(),
            handshakes: "handshakes".into(),
        }
    }
}