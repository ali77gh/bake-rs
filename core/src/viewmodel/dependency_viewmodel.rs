use std::{collections::HashMap, rc::Rc};

use crate::{
    model::dependency::Dependency,
    util::url::{generate_installation_link, standard_link},
};

use super::{capabilities::Capabilities, BakeViewModel};

/// Unknown if check commands is [None] and bake is not able to check if dependency is installed
#[derive(PartialEq)]
pub enum IsInstalledState {
    Installed,
    NotInstalled,
    Unknown,
}

/// contains [Dependency] yaml model but it actually contains logic (check and installation)
/// And use [Capabilities] for no side effects
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

    /// static function to generate hashmap for [BakeViewModel]
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

    /// checks if dependency is installed by running 'check' commands throw [Capabilities]
    pub fn is_installed(&self, bake_view_model: &BakeViewModel) -> IsInstalledState {
        let commands = match self.dependency.check() {
            Ok(commands) => commands,
            Err(_) => return IsInstalledState::Unknown,
        };

        match bake_view_model.run_commands(&commands) {
            Ok(_) => IsInstalledState::Installed,
            Err(_) => IsInstalledState::NotInstalled,
        }
    }

    /// to show in UI
    pub fn is_installable(&self) -> bool {
        self.dependency.installation_command().is_ok() || self.dependency.link().is_ok()
    }

    /// tries installing dependency by running installation commands
    /// skips if it's already installed
    /// if there is no installation commands for platform it tries to open link or generates installation link if its not specified
    /// Err on opening link
    pub fn try_install(&self, bake_view_model: &BakeViewModel) -> Result<(), String> {
        // TODO move dependencies of dependencies check and installation here from [BakeViewModel]
        if self.is_installed(bake_view_model) == IsInstalledState::Installed {
            return Ok(());
        }

        if let Ok(commands) = &self.dependency.installation_command() {
            bake_view_model.run_commands(commands)?;

            // TODO this check is duplicated in [BakeViewModel]
            match self.is_installed(bake_view_model) {
                IsInstalledState::Installed | IsInstalledState::Unknown => return Ok(()),
                IsInstalledState::NotInstalled => return Err(format!(
                    "'{}' installation ends without error but double check after installation failed",
                    self.name()
                )),
            }
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
