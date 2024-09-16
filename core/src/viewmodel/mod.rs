pub mod capabilities;
pub mod dependency_viewmodel;
pub mod message;
pub mod task_viewmodel;

use std::{rc::Rc, time::Instant};

use capabilities::Capabilities;
use dependency_viewmodel::{DependencyViewModel, IsInstalledState};
use message::Message;
use task_viewmodel::TaskViewModel;

use crate::model::bake_file::BakeFile;

const BAKE_FILE_NAME: &str = "bakefile.yaml";

pub struct BakeViewModel {
    dependencies: Vec<DependencyViewModel>,
    tasks: Vec<TaskViewModel>,
    caps: Rc<dyn Capabilities>,
}

impl BakeViewModel {
    /// returns None if file not bakefile exist
    pub fn new(caps: Rc<dyn Capabilities>) -> Result<Self, String> {
        if let Some(content) = caps.read_file(BAKE_FILE_NAME) {
            let bakefile = BakeFile::from_yaml(&content)?;
            let dependencies = bakefile
                .dependencies()
                .iter()
                .map(|dependency| DependencyViewModel::new(Rc::clone(&caps), dependency.clone()))
                .collect();

            let tasks = bakefile
                .tasks()
                .iter()
                .map(|task| TaskViewModel::new(Rc::clone(&caps), task.clone()))
                .collect();
            Ok(Self {
                caps,
                tasks,
                dependencies,
            })
        } else {
            Err("bakefile not found".to_string())
        }
    }

    pub fn dependencies(&self) -> &[DependencyViewModel] {
        &self.dependencies
    }

    pub fn tasks(&self) -> &[TaskViewModel] {
        &self.tasks
    }

    pub fn install_dependencies(&self, names: &[String]) -> Result<(), String> {
        for name in names {
            self.install_dependency(name)?;
        }
        Ok(())
    }

    pub fn install_dependency(&self, name: &str) -> Result<(), String> {
        let dependency = self
            .dependencies()
            .iter()
            .find(|dependency| dependency.name() == name);

        if let Some(dependency) = dependency {
            if dependency.is_installed() == IsInstalledState::Installed {
                return Ok(());
            }
            if !self.caps.ask_user_yes_no(
                format!("'{}' is not installed, do you want to install it", name).as_str(),
            ) {
                return Err(format!("cancel installation {}", name));
            }

            self.install_dependencies(dependency.dependencies())?;
            self.caps.message(Message::new(
                message::MessageType::BakeState,
                format!("dependency '{}' is installing...\n", dependency.name()),
            ));
            dependency.try_install()?;
            if dependency.is_installed() == IsInstalledState::NotInstalled {
                Err(format!("failed to install {}", name))
            } else {
                self.caps.message(Message::new(
                    message::MessageType::BakeState,
                    format!(
                        "dependency '{}' is installed successfully!\n",
                        dependency.name()
                    ),
                ));
                Ok(())
            }
        } else {
            Err(format!("dependency {} not found", name))
        }
    }

    pub fn run_task(&self, name: &str) -> Result<(), String> {
        let task = self.tasks().iter().find(|task| task.is(name));

        if let Some(task) = task {
            self.install_dependencies(task.dependencies())?;
            self.caps.message(Message::new(
                message::MessageType::BakeState,
                format!("task '{}' is running...\n", name),
            ));
            let start_time = Instant::now();
            task.run()?;
            let duration = start_time.elapsed();
            self.caps.message(Message::new(
                message::MessageType::BakeState,
                format!(
                    "Task '{}' finished successfully. time: {}ms\n",
                    name,
                    duration.as_millis()
                ),
            ));
            Ok(())
        } else {
            if let Ok(index) = name.parse::<usize>() {
                if let Some(task) = self.tasks.get(index - 1) {
                    return self.run_task(task.name());
                } else {
                    return Err(format!("task {} not found", name));
                }
            }
            Err(format!("task {} not found", name))
        }
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
plugins:
  - name: fs
    path: .bake/fs.yaml

global_env_vars:
  - name: PORT
    validation: Integer
    value: 80

dependencies:
  - name: rust
    dependencies: [ rust ]
    check: [ rustc --version ]
    link: https://somewhere.com
    command_linux: [ sudo apt install something ]

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

        fn execute_silent(&self, _: &str) -> Result<String, String> {
            Ok("done".to_string())
        }

        fn open_link(&self, _: &str) -> Result<(), String> {
            Ok(())
        }

        fn message(&self, input: Message) {
            print!("{}", input.content());
        }

        fn input(&self) -> String {
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer).unwrap();
            buffer
        }
    }

    #[test]
    fn test_name() {
        let cap = TestCap;
        let r = BakeViewModel::new(Rc::new(cap));
        assert_eq!(
            r.unwrap().tasks().get(0).unwrap().name(),
            "clean".to_string()
        );
    }
}
