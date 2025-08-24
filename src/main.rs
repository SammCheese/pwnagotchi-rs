#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::todo)]
#![warn(clippy::panic)]
#![warn(clippy::clone_on_copy)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::needless_collect)]
#![warn(clippy::single_match)]
#![warn(clippy::wildcard_imports)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]


extern crate pwnagotchi_rs;

use std::{time::Duration};

use pwnagotchi_rs::core::{agent::Agent, cli};
use tokio::time::sleep;
//use std::thread::sleep;


#[tokio::main]
async fn main() {
    let config = pwnagotchi_rs::core::config::Config::default();
    let mut agent = Agent::new(config);
    cli::do_auto_mode(&mut agent).await;

    loop {
        sleep(Duration::from_secs(60)).await;
    }
}

#[must_use]
pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION") 
}