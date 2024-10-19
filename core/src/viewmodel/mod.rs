pub mod capabilities;
pub mod dependency_viewmodel;
pub mod env_validator;
pub mod env_with_role_back;
pub mod message;
pub mod task_viewmodel;

use std::{collections::HashMap, rc::Rc};

use capabilities::Capabilities;
use dependency_viewmodel::{DependencyViewModel, IsInstalledState};
use env_validator::validate_envs;
use env_with_role_back::EnvWithRoleBack;
use message::Message;
use task_viewmodel::TaskViewModel;

use crate::{
    model::{bake_file::BakeFile, command::Command, plugin::Plugin},
    util::{measure_execution_time::measure_execution_time_result, ordered_map::OrderedMap},
};

const BAKE_FILE_NAME: &str = "bakefile.yaml";

pub struct BakeViewModel {
    plugins: HashMap<String, BakeViewModel>,
    dependencies: HashMap<String, DependencyViewModel>,
    tasks: OrderedMap<String, TaskViewModel>,
    caps: Rc<dyn Capabilities>,
}

impl BakeViewModel {
    /// returns None if bakefile not exist
    pub fn new_from_file(caps: Rc<dyn Capabilities>, file_name: &str) -> Result<Self, String> {
        if let Some(content) = caps.read_file(file_name) {
            let bakefile = BakeFile::from_yaml(&content)?;
            Ok(Self {
                caps: Rc::clone(&caps),
                plugins: Self::from_plugins(Rc::clone(&caps), bakefile.plugins())?,
                dependencies: DependencyViewModel::hashmap_from_dependencies(
                    Rc::clone(&caps),
                    bakefile.dependencies(),
                ),
                tasks: TaskViewModel::hashmap_from_tasks(Rc::clone(&caps), bakefile.tasks()),
            })
        } else {
            Err(format!("{} not found", file_name))
        }
    }

    pub fn from_plugin(caps: Rc<dyn Capabilities>, plugin: &Plugin) -> Result<Self, String> {
        Self::new_from_file(caps, plugin.path())
    }

    pub fn from_plugins(
        caps: Rc<dyn Capabilities>,
        plugins: &[Plugin],
    ) -> Result<HashMap<String, Self>, String> {
        let mut hashmap = HashMap::with_capacity(plugins.len());
        for plugin in plugins {
            hashmap.insert(
                plugin.name().to_string(),
                Self::new_from_file(Rc::clone(&caps), plugin.path())?,
            );
        }
        Ok(hashmap)
    }

    pub fn new(caps: Rc<dyn Capabilities>) -> Result<Self, String> {
        Self::new_from_file(caps, BAKE_FILE_NAME)
    }

    pub fn get_dependency(&self, name: &str) -> Option<&DependencyViewModel> {
        self.dependencies.get(name)
    }

    pub fn get_task(&self, name: &str) -> Option<&TaskViewModel> {
        self.tasks.get(&name.to_string())
    }

    pub fn get_task_at(&self, index: usize) -> Option<&TaskViewModel> {
        self.tasks.get_at(index)
    }

    pub fn tasks(&self) -> &[TaskViewModel] {
        self.tasks.get_all()
    }

    pub fn dependencies(&self) -> Vec<&DependencyViewModel> {
        self.dependencies.values().collect()
    }

    pub fn install_dependencies(&self, names: &[String]) -> Result<(), String> {
        for name in names {
            self.install_dependency(name)?;
        }
        Ok(())
    }

    pub fn install_dependency(&self, name: &str) -> Result<(), String> {
        if let Some(dependency) = self.get_dependency(name) {
            if dependency.is_installed(self) == IsInstalledState::Installed {
                return Ok(());
            }
            // auto yes if can't get user input
            if let Some(false) = self.caps.ask_user_yes_no(
                format!("'{}' is not installed, do you want to install it", name).as_str(),
            ) {
                return Err(format!("cancel installation {}", name));
            }

            self.install_dependencies(dependency.dependencies())?;
            self.caps.message(Message::bake_state(format!(
                "dependency '{}' is installing...\n",
                dependency.name()
            )));
            dependency.try_install(self)?;
            if dependency.is_installed(self) == IsInstalledState::NotInstalled {
                Err(format!("failed to install {}", name))
            } else {
                self.caps.message(Message::bake_state(format!(
                    "dependency '{}' is installed successfully!\n",
                    dependency.name()
                )));
                Ok(())
            }
        } else {
            Err(format!("dependency {} not found", name))
        }
    }

    pub fn run_task(&self, name: &str) -> Result<(), String> {
        if let Some(task) = self.get_task(name) {
            self.install_dependencies(task.dependencies())?;
            validate_envs(Rc::clone(&self.caps), task)?;
            self.caps.message(Message::bake_state(format!(
                "task '{name}' is running...\n"
            )));
            let (_, duration) = measure_execution_time_result(|| task.run(self))?;
            self.caps.message(Message::bake_state(format!(
                "Task '{name}' finished successfully. time: {}ms\n",
                duration.as_millis()
            )));
            Ok(())
        } else {
            let index = name
                .parse::<usize>()
                .or(Err(format!("task {} not found", name)))?;
            let task = self
                .get_task_at(index - 1)
                .ok_or(format!("task {} not found", name))?;
            self.run_task(task.name())
        }
    }

    pub fn run_command(&self, command: &Command) -> Result<(), String> {
        match command {
            Command::ShellCommand(cmd) => {
                if self.caps.execute(cmd) {
                    Ok(())
                } else {
                    Err("".to_string())
                }
            }
            Command::FunctionCall(fc) => match fc.namespace() {
                "this" => {
                    let mut env_with_role_back = EnvWithRoleBack::new();
                    env_with_role_back.set_envs(fc.params());
                    let r = self.run_task(fc.function());
                    env_with_role_back.role_back();
                    r
                }
                namespace => match self.plugins.get(namespace) {
                    Some(x) => {
                        let mut env_with_role_back = EnvWithRoleBack::new();
                        env_with_role_back.set_envs(fc.params());
                        let r = x.run_task(fc.function());
                        env_with_role_back.role_back();
                        r
                    }
                    None => Err(format!("namespace '{}' not found", namespace)),
                },
            },
        }
    }

    pub fn run_commands(&self, commands: &[Command]) -> Result<(), String> {
        for command in commands {
            self.run_command(command)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use message::Message;

    use super::*;

    struct TestCap;
    impl Capabilities for TestCap {
        fn read_file(&self, file_name: &str) -> Option<String> {
            if BAKE_FILE_NAME != file_name {
                return None;
            }
            Some(
                "
# plugins:
#   - name: fs
#     path: .bake/fs.yaml

global_env_vars:
  - name: PORT
    validation: Integer
    value: 80

dependencies:
  - name: rust
    dependencies: [ rust ]
    check: [ rustc --version ]
    link: https://somewhere.com
    commands_linux: [ sudo apt install something ]

tasks:
  - name: clean
    help_msg: removes some stuff
    dependencies: [ rust ]
    params:
      - name: USERNAME
    commands:
      - cmd1
      - cmd2
"
                .to_string(),
            )
        }

        fn execute(&self, _: &str) -> bool {
            Message::normal("done");
            true
        }

        fn open_link(&self, _: &str) -> Result<(), String> {
            Ok(())
        }

        fn message(&self, input: Message) {
            print!("{}", input.content());
        }

        fn input(&self) -> Option<String> {
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer).unwrap();
            Some(buffer)
        }
    }

    #[test]
    fn test_name() {
        let cap = TestCap;
        let r = BakeViewModel::new(Rc::new(cap));
        assert_eq!(
            r.unwrap().get_task_at(0).unwrap().name(),
            "clean".to_string()
        );
    }
}
