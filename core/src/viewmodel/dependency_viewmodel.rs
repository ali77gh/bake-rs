use std::{collections::HashMap, rc::Rc};

use crate::{
    model::dependency::Dependency,
    util::url::{generate_installation_link, standard_link},
};

use super::{capabilities::Capabilities, BakeViewModel};

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

impl DependencyViewModel {
    pub fn new(capabilities: Rc<dyn Capabilities>, dependency: Dependency) -> Self {
        Self {
            capabilities,
            dependency,
        }
    }

    pub fn hashmap_from_dependencies(
        capabilities: Rc<dyn Capabilities>,
        dependencies: &[Dependency],
    ) -> HashMap<String, DependencyViewModel> {
        dependencies
            .iter()
            .map(|dependency| {
                (
                    dependency.name().to_string(),
                    DependencyViewModel::new(Rc::clone(&capabilities), dependency.clone()),
                )
            })
            .collect::<HashMap<String, DependencyViewModel>>()
    }

    pub fn name(&self) -> &str {
        self.dependency.name()
    }

    pub fn dependencies(&self) -> &[String] {
        self.dependency.dependencies()
    }

    pub fn is_installed(&self, bake_view_model: &BakeViewModel) -> IsInstalledState {
        let commands = match self.dependency.check() {
            Ok(commands) => commands,
            Err(_) => return IsInstalledState::Unknown,
        };

        // check commands runs silently
        match bake_view_model.run_commands(&commands) {
            Ok(_) => IsInstalledState::Installed,
            Err(_) => IsInstalledState::NotInstalled,
        }
    }

    pub fn is_installable(&self) -> bool {
        self.dependency.installation_command().is_ok() || self.dependency.link().is_ok()
    }

    pub fn try_install(&self, bake_view_model: &BakeViewModel) -> Result<(), String> {
        if self.is_installed(bake_view_model) == IsInstalledState::Installed {
            return Ok(());
        }

        if let Ok(commands) = &self.dependency.installation_command() {
            bake_view_model.run_commands(commands)?;
            return Ok(());
        }

        if let Ok(link) = &self.dependency.link() {
            self.capabilities.open_link(&standard_link(link));
            return Ok(());
        }

        // auto yes if none
        if let Some(true) | None = self.capabilities.ask_user_yes_no(
            format!(
                "There is no installation command or link for '{}' Do you want to search for it on google",
                self.dependency.name()
            ).as_str()
        ){
            self.capabilities.open_link(&generate_installation_link(self.name()));
        }

        Err(format!("{} is not installable", self.name()))
    }
}
