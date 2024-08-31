use serde::{Deserialize, Serialize};

use super::{dependency::Dependency, param::Param, task::Task};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct BakeFile {
    global_env_vars: Option<Vec<Param>>,
    dependencies: Option<Vec<Dependency>>,
    tasks: Option<Vec<Task>>,
}

impl BakeFile {
    pub fn from_yaml(yaml: &str) -> Result<BakeFile, String> {
        match serde_yaml::from_str::<BakeFile>(yaml) {
            Ok(x) => Ok(x),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }

    pub fn global_env_vars(&self) -> &Vec<Param> {
        const EMPTY: &'static Vec<Param> = &vec![];
        if let Some(x) = &self.global_env_vars {
            return x;
        } else {
            return EMPTY;
        }
    }

    pub fn dependencies(&self) -> &Vec<Dependency> {
        const EMPTY: &'static Vec<Dependency> = &vec![];
        if let Some(x) = &self.dependencies {
            return x;
        } else {
            return EMPTY;
        }
    }

    pub fn tasks(&self) -> &Vec<Task> {
        const EMPTY: &'static Vec<Task> = &vec![];
        if let Some(x) = &self.tasks {
            return x;
        } else {
            return EMPTY;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::param::ParamValidation;

    #[test]
    fn yaml_generator_test() {
        let task = Task::new(
            "clean".to_string(),
            Some("removes some stuff".to_string()),
            Some(vec!["rust".to_string()]),
            Some(vec![Param::new(
                "PORT".to_string(),
                Some(ParamValidation::Integer),
            )]),
            Some(vec!["cmd1".to_string(), "cmd2".to_string()]),
            None,
            None,
            None,
        );

        let bake_file = BakeFile {
            global_env_vars: Some(vec![Param::new("USERNAME".to_string(), None)]),
            dependencies: Some(vec![Dependency::new(
                "rust".to_string(),
                Some(vec!["rust-dependency".to_string()]),
                Some(vec!["rustc --version".to_string()]),
                None,
                None,
                None,
                Some(vec!["https://somewhere.com".to_string()]),
                None,
                None,
                None,
                None,
                Some(vec!["sudo apt install something".to_string()]),
                None,
                None,
            )]),
            tasks: Some(vec![task]),
        };
        let yaml = bake_file.to_yaml();
        dbg!(yaml);
        // assert!(false);
    }

    #[test]
    fn yaml_parser_test() {
        let yaml = "
global_env_vars:
- name: PORT
  validation: Integer
  value: 80

dependencies:
- name: rust
  dependencies: [ rust ]
  check: [ rustc --version ]
  link: [ https://somewhere.com ]
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
";
        let bake_file = BakeFile::from_yaml(yaml).unwrap();

        dbg!(bake_file);
        // assert!(false);
    }
}
