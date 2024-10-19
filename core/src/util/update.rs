use crate::viewmodel::capabilities::Capabilities;

pub const UPDATE_URL: &str = "https://github.com/ali77gh/bake-rs";

pub fn update(capabilities: &dyn Capabilities) {
    capabilities.open_link(UPDATE_URL);
}
