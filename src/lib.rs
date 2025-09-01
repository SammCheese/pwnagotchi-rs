#![allow(dead_code)]
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

pub mod core {
    pub mod agent;
    pub mod ai;
    pub mod automata;
    pub mod bettercap;
    pub mod cli;
    pub mod commands;
    pub mod config;
    pub mod events;
    pub mod identity;
    pub mod log;
    pub mod mesh;
    pub mod models;
    pub mod session;
    pub mod stats;
    pub mod ui;
    pub mod utils;
    pub mod voice;
}

mod traits {
    pub mod hostname;
    pub mod logger;
    pub mod syscontrol;
    pub mod sysdata;
}

mod net {}
