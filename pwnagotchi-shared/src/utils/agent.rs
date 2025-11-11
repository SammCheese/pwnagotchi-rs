use parking_lot::RwLock;

use crate::{models::net::AccessPoint, sessions::session::Session};

pub fn get_aps_on_channel(session: &RwLock<Session>, channel: u8) -> Vec<AccessPoint> {
  session
    .read()
    .state
    .access_points
    .iter()
    .filter(|ap| ap.channel == channel)
    .cloned()
    .collect()
}
