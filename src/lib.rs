#![allow(dead_code)]

mod core {
    mod config;
    mod bettercap;
    mod log;
    mod agent;
    mod identity;
}

mod traits {
    pub mod sysdata;
    pub mod syscontrol;
    pub mod hostname;
    pub mod logger;
}

mod net {

}
