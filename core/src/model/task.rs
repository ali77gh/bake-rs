use super::{command::Command, dependency::Dependency};

pub struct Task {
    name: String,
    help_msg: Option<String>,
    dependencies: Vec<Dependency>,
    // TODO params
    commands: Vec<Command>,
    commands_linux_only: Vec<Command>,
    commands_mac_only: Vec<Command>,
    commands_win_only: Vec<Command>,
}
