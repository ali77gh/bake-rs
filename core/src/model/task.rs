use serde::{Deserialize, Serialize};

use super::param::Param;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Task {
    name: String,
    help_msg: Option<String>,
    dependencies: Option<Vec<String>>,
    params: Option<Vec<Param>>,

    commands: Option<Vec<String>>,
    commands_linux: Option<Vec<String>>,
    commands_windows: Option<Vec<String>>,
    commands_macos: Option<Vec<String>>,
}

impl Task {
    pub fn new(
        name: String,
        help_msg: Option<String>,
        dependencies: Option<Vec<String>>,
        params: Option<Vec<Param>>,
        commands: Option<Vec<String>>,
        commands_linux: Option<Vec<String>>,
        commands_windows: Option<Vec<String>>,
        commands_macos: Option<Vec<String>>,
    ) -> Self {
        Self {
            name,
            help_msg,
            dependencies,
            params,
            commands,
            commands_linux,
            commands_windows,
            commands_macos,
        }
    }
}
