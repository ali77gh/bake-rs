use serde::Deserialize;

use crate::util::platform_specific::{get_platform_name, platform_specific};

use super::{command::Command, param::Param};

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Task {
    name: String,
    help_msg: Option<String>,
    dependencies: Option<Vec<String>>,
    envs: Option<Vec<Param>>,

    commands: Option<Vec<String>>,
    commands_linux: Option<Vec<String>>,
    commands_windows: Option<Vec<String>>,
    commands_macos: Option<Vec<String>>,
}

#[allow(clippy::too_many_arguments)]
impl Task {
    pub fn new(
        name: String,
        help_msg: Option<String>,
        dependencies: Option<Vec<String>>,
        envs: Option<Vec<Param>>,
        commands: Option<Vec<String>>,
        commands_linux: Option<Vec<String>>,
        commands_windows: Option<Vec<String>>,
        commands_macos: Option<Vec<String>>,
    ) -> Self {
        Self {
            name,
            help_msg,
            dependencies,
            envs,
            commands,
            commands_linux,
            commands_windows,
            commands_macos,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn help_msg(&self) -> Option<&str> {
        self.help_msg.as_deref()
    }

    pub fn dependencies(&self) -> &[String] {
        const EMPTY: &Vec<String> = &vec![];
        if let Some(x) = &self.dependencies {
            x
        } else {
            EMPTY
        }
    }

    pub fn envs(&self) -> &[Param] {
        const EMPTY: &Vec<Param> = &vec![];
        if let Some(x) = &self.envs {
            x
        } else {
            EMPTY
        }
    }

    pub fn commands(&self) -> Result<Vec<Command>, String> {
        let commands = platform_specific(
            self.commands.as_ref(),
            self.commands_linux.as_ref(),
            self.commands_windows.as_ref(),
            self.commands_macos.as_ref(),
        );

        if let Some(commands) = commands {
            commands
                .iter()
                .map(|x| Command::try_from(x.as_str()))
                .collect::<Result<Vec<Command>, String>>()
        } else {
            Err(format!("{} is not supported", get_platform_name()))
        }
    }
}
