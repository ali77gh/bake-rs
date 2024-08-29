use super::{dependency::Dependency, param::Param, platform_specific::PlatformSpecificCommands};

pub struct Task {
    name: String,
    help_msg: Option<String>,
    dependencies: Vec<Dependency>,
    params: Vec<Param>,
    commands: PlatformSpecificCommands,
}
