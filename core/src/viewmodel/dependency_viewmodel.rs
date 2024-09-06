use std::rc::Rc;

use crate::model::{command::Command, dependency::Dependency};

use super::capabilities::Capabilities;

pub enum IsInstalledState {
    Installed,
    NotInstalled,
    Unknown,
}

pub struct DependencyViewModel {
    capabilities: Rc<dyn Capabilities>,
    dependency: Dependency,
}

impl DependencyViewModel {
    pub fn new(capabilities: Rc<dyn Capabilities>, dependency: Dependency) -> Self {
        Self {
            capabilities,
            dependency,
        }
    }

    pub fn name(&self) -> &str {
        &self.dependency.name()
    }

    pub fn is_installed(&self) -> IsInstalledState {
        let commands = match self.dependency.check() {
            Ok(commands) => commands,
            Err(_) => return IsInstalledState::Unknown,
        };

        //TODO remove this filter (you should run function calls too!)
        let commands = filter_commands(&commands);

        match self.capabilities.execute_all(&commands) {
            Ok(_) => IsInstalledState::Installed,
            Err(_) => IsInstalledState::NotInstalled,
        }
    }

    pub fn is_installable(&self) -> bool {
        self.dependency.installation_command().is_ok() || self.dependency.link().is_ok()
    }

    pub fn try_install(&self) -> Result<(), String> {
        if let Ok(commands) = &self.dependency.installation_command() {
            //TODO remove this filter (you should run function calls too!)
            let commands = filter_commands(&commands);
            self.capabilities.execute_all(&commands)?;
            return Ok(());
        }

        if let Ok(link) = &self.dependency.link() {
            self.capabilities.open_link(&link)?;
            return Ok(());
        }

        Err("not installable".to_string())
    }
}

//TODO remove this filter (you should run function calls too!)
fn filter_commands(commands: &Vec<Command>) -> Vec<&str> {
    commands
        .iter()
        .filter_map(|command| match command {
            crate::model::command::Command::ShellCommand(s) => Some(s.as_str()),
            crate::model::command::Command::FunctionCall(_) => None,
        })
        .collect::<Vec<&str>>()
}
