use serde::Deserialize;

use crate::util::platform_specific::{get_platform_name, platform_specific};

use super::command::Command;

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Dependency {
    name: String,
    dependencies: Option<Vec<String>>,

    check: Option<Vec<String>>,
    check_linux: Option<Vec<String>>,
    check_windows: Option<Vec<String>>,
    check_macos: Option<Vec<String>>,

    link: Option<String>,
    link_linux: Option<String>,
    link_windows: Option<String>,
    link_macos: Option<String>,

    commands: Option<Vec<String>>,
    commands_linux: Option<Vec<String>>,
    commands_windows: Option<Vec<String>>,
    commands_macos: Option<Vec<String>>,
}

impl Dependency {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dependencies(&self) -> &[String] {
        if let Some(dependencies) = &self.dependencies {
            dependencies
        } else {
            &[]
        }
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
                .map(|x| Command::try_from(x.as_str()))
                .collect::<Result<Vec<Command>, String>>()
        } else {
            Err(format!("{} is not supported", get_platform_name()))
        }
    }

    pub fn link(&self) -> Result<String, String> {
        platform_specific(
            self.link.as_ref(),
            self.link_linux.as_ref(),
            self.link_windows.as_ref(),
            self.link_macos.as_ref(),
        )
        .ok_or(format!("no command available for {}", get_platform_name()))
        .cloned()
    }

    pub fn installation_command(&self) -> Result<Vec<Command>, String> {
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

#[allow(clippy::too_many_arguments)]
impl Dependency {
    pub fn new(
        name: String,
        dependencies: Option<Vec<String>>,
        check: Option<Vec<String>>,
        check_linux: Option<Vec<String>>,
        check_windows: Option<Vec<String>>,
        check_macos: Option<Vec<String>>,
        link: Option<String>,
        link_linux: Option<String>,
        link_windows: Option<String>,
        link_macos: Option<String>,
        commands: Option<Vec<String>>,
        commands_linux: Option<Vec<String>>,
        commands_windows: Option<Vec<String>>,
        commands_macos: Option<Vec<String>>,
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
            commands,
            commands_linux,
            commands_windows,
            commands_macos,
        }
    }
}
