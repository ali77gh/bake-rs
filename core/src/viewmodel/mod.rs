pub mod capabilities;
pub mod dependency_viewmodel;
pub mod task_viewmodel;

use std::rc::Rc;

use capabilities::Capabilities;
use dependency_viewmodel::DependencyViewModel;
use task_viewmodel::TaskViewModel;

use crate::model::bake_file::BakeFile;

const BAKE_FILE_NAME: &str = "bakefile.yaml";

pub struct BakeViewModel {
    bake_file: BakeFile,

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
                bake_file: bakefile,
                caps,
                tasks,
                dependencies,
            })
        } else {
            Err("bakefile not found".to_string())
        }
    }

    ///
    pub fn bake_file(&self) -> &BakeFile {
        &self.bake_file
    }
}

#[cfg(test)]
mod tests {
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

        fn execute(&self, _: &str) -> Result<String, String> {
            Ok("done".to_string())
        }

        fn open_link(&self, _: &str) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_name() {
        let cap = TestCap;
        let r = BakeViewModel::new(Rc::new(cap));
        assert_eq!(
            r.unwrap().bake_file().tasks().get(0).unwrap().name(),
            "clean".to_string()
        );
    }
}
