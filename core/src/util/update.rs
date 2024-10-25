use crate::viewmodel::capabilities::Capabilities;

pub const UPDATE_URL: &str = "https://github.com/ali77gh/bake-rs";

/// it tries to open [UPDATE_URL]
pub fn update(capabilities: &dyn Capabilities) {
    capabilities.open_link(UPDATE_URL);
}
