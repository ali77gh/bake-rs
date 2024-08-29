use super::command::Command;

pub struct Dependency {
    name: String,

    dependencies: Vec<Dependency>,

    check_command: Option<Command>,
    check_command_linux: Option<Command>,
    check_command_macos: Option<Command>,
    check_command_windows: Option<Command>,

    install_link: Option<String>,
    install_link_linux: Option<String>,
    install_link_macos: Option<String>,
    install_link_windows: Option<String>,

    install_command: Option<String>,
    install_command_linux: Option<String>,
    install_command_macos: Option<String>,
    install_command_windows: Option<String>,
}
