pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn show_version() {
    println!("{}", VERSION);
}
