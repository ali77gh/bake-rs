use crate::viewmodel::{capabilities::Capabilities, message::Message};

pub const UPDATE_URL: &str = "https://github.com/ali77gh/bake-rs";

pub fn update(capabilities: &dyn Capabilities) {
    if let Err(e) = capabilities.open_link(UPDATE_URL) {
        capabilities.message(Message::error(e));
        println!(
            "error while opening web browser!\n check here for update: {}",
            UPDATE_URL
        );
    }
}
