use serde::{Deserialize, Serialize};

use crate::util::platform_specific::{get_platform_name, platform_specific};

use super::command::Command;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Dependency {
    name: String,
    dependencies: Option<Vec<String>>,

    check: Option<Vec<String>>,
    check_linux: Option<Vec<String>>,
    check_windows: Option<Vec<String>>,
    check_macos: Option<Vec<String>>,

    link: Option<Vec<String>>,
    link_linux: Option<Vec<String>>,
    link_windows: Option<Vec<String>>,
    link_macos: Option<Vec<String>>,

    command: Option<Vec<String>>,
    command_linux: Option<Vec<String>>,
    command_windows: Option<Vec<String>>,
    command_macos: Option<Vec<String>>,
}

impl Dependency {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dependencies(&self) -> &Option<Vec<String>> {
        &self.dependencies
    }

    pub fn check(&self) -> Result<Vec<Command>, String> {
        let commands = platform_specific(
            self.check.as_ref(),
            self.check_linux.as_ref(),
            self.check_windows.as_ref(),
            self.check_macos.as_ref(),
        );

        if let Some(commands) = commands {
            commands
                .iter()
                .map(|str| Command::from_str(str))
                .collect::<Result<Vec<Command>, String>>()
        } else {
            return Err(format!("{} is not supported", get_platform_name()));
        }
    }

    pub fn link(&self) -> Result<Vec<String>, String> {
        let links = platform_specific(
            self.link.as_ref(),
            self.link_linux.as_ref(),
            self.link_windows.as_ref(),
            self.link_macos.as_ref(),
        );

        if let Some(links) = links {
            return Ok(links.to_vec());
        } else {
            return Err(format!("{} is not supported", get_platform_name()));
        }
    }

    pub fn installation_command(&self) -> Result<Vec<Command>, String> {
        let commands = platform_specific(
            self.command.as_ref(),
            self.command_linux.as_ref(),
            self.command_windows.as_ref(),
            self.command_macos.as_ref(),
        );

        if let Some(commands) = commands {
            commands
                .iter()
                .map(|str| Command::from_str(str))
                .collect::<Result<Vec<Command>, String>>()
        } else {
            return Err(format!("{} is not supported", get_platform_name()));
        }
    }
}

impl Dependency {
    pub fn new(
        name: String,
        dependencies: Option<Vec<String>>,
        check: Option<Vec<String>>,
        check_linux: Option<Vec<String>>,
        check_windows: Option<Vec<String>>,
        check_macos: Option<Vec<String>>,
        link: Option<Vec<String>>,
        link_linux: Option<Vec<String>>,
        link_windows: Option<Vec<String>>,
        link_macos: Option<Vec<String>>,
        command: Option<Vec<String>>,
        command_linux: Option<Vec<String>>,
        command_windows: Option<Vec<String>>,
        command_macos: Option<Vec<String>>,
    ) -> Self {
        Self {
            name,
            dependencies,
            check,
            check_linux,
            check_windows,
            check_macos,
            link,
            link_linux,
            link_windows,
            link_macos,
            command,
            command_linux,
            command_windows,
            command_macos,
        }
    }
}
