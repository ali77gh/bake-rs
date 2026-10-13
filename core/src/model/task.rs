use std::fmt::Display;

use serde::Deserialize;

use crate::util::platform_specific::{get_platform_name, platform_specific};

use super::{command::Command, function_call::FunctionCall, param::Param};

pub type TaskStatusCode = i32;

#[derive(Debug, PartialEq)]
pub struct TaskError {
    pub message: String,
    pub code: TaskStatusCode,
}

impl TaskError {
    pub fn new(message: String, code: TaskStatusCode) -> Self {
        Self { message, code }
    }
}

impl From<String> for TaskError {
    fn from(value: String) -> Self {
        Self::new(value, 1)
    }
}

impl Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

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

    keep_alive: Option<bool>,
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
        keep_alive: Option<bool>,
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
            keep_alive,
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
    pub fn commands(&self) -> Result<Vec<Command>, TaskError> {
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
                .collect::<Result<Vec<Command>, TaskError>>()
        } else {
            Err(format!("{} is not supported", get_platform_name()).into())
        }
    }

    pub fn on_success(&self) -> Option<Result<FunctionCall, TaskError>> {
        self.on_success
            .as_ref()
            .map(|value| FunctionCall::try_from(value.as_str()))
    }

    pub fn on_error(&self) -> Option<Result<FunctionCall, TaskError>> {
        self.on_error
            .as_ref()
            .map(|value| FunctionCall::try_from(value.as_str()))
    }

    pub fn on_end(&self) -> Option<Result<FunctionCall, TaskError>> {
        self.on_end
            .as_ref()
            .map(|value| FunctionCall::try_from(value.as_str()))
    }

    pub fn working_directory(&self) -> Option<&str> {
        self.working_directory.as_deref()
    }

    pub fn keep_alive(&self) -> bool {
        match self.keep_alive {
            Some(false) => false,
            Some(true) => true,
            None => false,
        }
    }
}

#[cfg(test)]
mod task_error_tests {
    use super::*;

    #[test]
    fn from_string_defaults_to_code_1() {
        let e: TaskError = "boom".to_string().into();
        assert_eq!(e.code, 1);
        assert_eq!(e.message, "boom");
    }

    #[test]
    fn display_prints_message_only() {
        assert_eq!(TaskError::new("boom".to_string(), 7).to_string(), "boom");
    }
}
