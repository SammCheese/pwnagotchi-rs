use std::sync::Arc;

use crate::{models::net::AccessPoint, sessions::session::Session};

pub fn get_aps_on_channel(session: &Arc<Session>, channel: u8) -> Vec<AccessPoint> {
  session
    .state
    .read()
    .access_points
    .iter()
    .filter(|ap| ap.channel == channel)
    .cloned()
    .collect()
}
