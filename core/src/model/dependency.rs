use super::platform_specific::{PlatformSpecificCommands, PlatformSpecificURLs};

pub struct Dependency {
    name: String,
    dependencies: Vec<Dependency>,

    check_command: PlatformSpecificCommands,
    install_link: PlatformSpecificURLs,
    install_command: PlatformSpecificCommands,
}
