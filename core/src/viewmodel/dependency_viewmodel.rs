use std::rc::Rc;

use crate::{
    model::{command::Command, dependency::Dependency},
    util::platform_specific,
};

use super::capabilities::Capabilities;

#[derive(PartialEq)]
pub enum IsInstalledState {
    Installed,
    NotInstalled,
    Unknown,
}

pub struct DependencyViewModel {
    capabilities: Rc<dyn Capabilities>,
    dependency: Dependency,
}

// TODO install dependencies of dependencies
impl DependencyViewModel {
    pub fn new(capabilities: Rc<dyn Capabilities>, dependency: Dependency) -> Self {
        Self {
            capabilities,
            dependency,
        }
    }

    pub fn name(&self) -> &str {
        self.dependency.name()
    }

    pub fn dependencies(&self) -> &[String] {
        self.dependency.dependencies()
    }

    pub fn is_installed(&self) -> IsInstalledState {
        let commands = match self.dependency.check() {
            Ok(commands) => commands,
            Err(_) => return IsInstalledState::Unknown,
        };

        //TODO remove this filter (you should run function calls too!)
        let commands = filter_commands(&commands);

        // check commands runs silently
        match self.capabilities.execute_silent_all(&commands) {
            Ok(_) => IsInstalledState::Installed,
            Err(_) => IsInstalledState::NotInstalled,
        }
    }

    pub fn is_installable(&self) -> bool {
        self.dependency.installation_command().is_ok() || self.dependency.link().is_ok()
    }

    pub fn try_install(&self) -> Result<(), String> {
        if self.is_installed() == IsInstalledState::Installed {
            return Ok(());
        }

        if let Ok(commands) = &self.dependency.installation_command() {
            //TODO remove this filter (you should run function calls too!)
            let commands = filter_commands(commands);
            self.capabilities.execute_and_print_all(&commands)?;
            return Ok(());
        }

        if let Ok(link) = &self.dependency.link() {
            self.capabilities.open_link(&standard_link(link))?;
            return Ok(());
        }

        if self.capabilities.ask_user_yes_no(
            format!(
                "There is no installation command or link for '{}' Do you want to search for it on google",
                self.dependency.name()
            ).as_str()
        ){
            let _ = self.capabilities.open_link(&standard_link(
                format!(
                    "https://google.com/search?q=how+to+install+{}+on+{}",
                    self.name(),
                    platform_specific::get_platform_name()
                )
                .as_str(),
            ));
        }

        Err(format!("{} is not installable", self.name()))
    }
}

fn standard_link(url: &str) -> String {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        format!("http://{}", url)
    } else {
        url.to_owned()
    }
}

//TODO remove this filter (you should run function calls too!)
pub fn filter_commands(commands: &[Command]) -> Vec<&str> {
    commands
        .iter()
        .filter_map(|command| match command {
            crate::model::command::Command::ShellCommand(s) => Some(s.as_str()),
            crate::model::command::Command::FunctionCall(_) => None,
        })
        .collect::<Vec<&str>>()
}
