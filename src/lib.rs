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
    pub mod config;
    pub mod bettercap;
    pub mod log;
    pub mod agent;
    pub mod identity;
    pub mod utils;
    pub mod ai;
    pub mod mesh;
    pub mod automata;
    pub mod cli;
    pub mod models;
    pub mod ui;
    pub mod events;
}

mod traits {
    pub mod sysdata;
    pub mod syscontrol;
    pub mod hostname;
    pub mod logger;
}

mod net {

}
