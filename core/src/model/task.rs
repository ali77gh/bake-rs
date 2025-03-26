use serde::Deserialize;

use crate::util::platform_specific::{get_platform_name, platform_specific};

use super::{command::Command, end_handler::EndHandler, param::Param};

/// yaml model
#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Task {
    name: String,
    help_msg: Option<String>,
    dependencies: Option<Vec<String>>,
    envs: Option<Vec<Param>>,
    working_directory: Option<String>,

    commands: Option<Vec<String>>,
    commands_linux: Option<Vec<String>>,
    commands_windows: Option<Vec<String>>,
    commands_macos: Option<Vec<String>>,

    on_success: Option<String>,
    on_error: Option<String>,
    on_end: Option<String>,
}

/// factory function and getters
#[allow(clippy::too_many_arguments)]
impl Task {
    pub fn new(
        name: String,
        help_msg: Option<String>,
        dependencies: Option<Vec<String>>,
        envs: Option<Vec<Param>>,
        working_directory: Option<String>,
        commands: Option<Vec<String>>,
        commands_linux: Option<Vec<String>>,
        commands_windows: Option<Vec<String>>,
        commands_macos: Option<Vec<String>>,
        on_success: Option<String>,
        on_error: Option<String>,
        on_end: Option<String>,
    ) -> Self {
        Self {
            name,
            help_msg,
            dependencies,
            envs,
            working_directory,
            commands,
            commands_linux,
            commands_windows,
            commands_macos,
            on_success,
            on_error,
            on_end,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn help_msg(&self) -> Option<&str> {
        self.help_msg.as_deref()
    }

    pub fn dependencies(&self) -> &[String] {
        if let Some(x) = &self.dependencies {
            x
        } else {
            &[]
        }
    }

    pub fn envs(&self) -> &[Param] {
        if let Some(x) = &self.envs {
            x
        } else {
            &[]
        }
    }

    // TODO return enum as error with Error trait impl
    /// ## Returns
    /// list of commands to do the task
    ///
    /// ## Errors:
    /// - command parser error
    /// - command not found on platform
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

    pub fn on_success(&self) -> Option<Result<EndHandler, String>> {
        self.on_success
            .as_ref()
            .map(|value| EndHandler::try_from(value.as_str()))
    }

    pub fn on_error(&self) -> Option<Result<EndHandler, String>> {
        self.on_error
            .as_ref()
            .map(|value| EndHandler::try_from(value.as_str()))
    }

    pub fn on_end(&self) -> Option<Result<EndHandler, String>> {
        self.on_end
            .as_ref()
            .map(|value| EndHandler::try_from(value.as_str()))
    }

    pub fn working_directory(&self) -> Option<&str> {
        self.working_directory.as_deref()
    }
}
