
fn main() {
    println!("Pwnagotchi version: {}", version());
}

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION") 
}