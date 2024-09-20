use serde::{Deserialize, Serialize};

use super::{dependency::Dependency, param::Param, plugin::Plugin, task::Task};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BakeFile {
    plugins: Option<Vec<Plugin>>,
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

    pub fn plugins(&self) -> &[Plugin] {
        const EMPTY: &Vec<Plugin> = &vec![];
        if let Some(x) = &self.plugins {
            x
        } else {
            EMPTY
        }
    }

    pub fn global_env_vars(&self) -> &[Param] {
        const EMPTY: &Vec<Param> = &vec![];
        if let Some(x) = &self.global_env_vars {
            x
        } else {
            EMPTY
        }
    }

    pub fn dependencies(&self) -> &[Dependency] {
        const EMPTY: &Vec<Dependency> = &vec![];
        if let Some(x) = &self.dependencies {
            x
        } else {
            EMPTY
        }
    }

    pub fn tasks(&self) -> &[Task] {
        const EMPTY: &Vec<Task> = &vec![];
        if let Some(x) = &self.tasks {
            x
        } else {
            EMPTY
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
                Some(ParamValidation::integer),
            )]),
            Some(vec!["cmd1".to_string(), "cmd2".to_string()]),
            None,
            None,
            None,
        );

        let bake_file = BakeFile {
            plugins: Some(vec![Plugin::new(
                "fs".to_string(),
                ".bake/fs.yaml".to_string(),
            )]),
            global_env_vars: Some(vec![Param::new("USERNAME".to_string(), None)]),
            dependencies: Some(vec![Dependency::new(
                "rust".to_string(),
                Some(vec!["rust-dependency".to_string()]),
                Some(vec!["rustc --version".to_string()]),
                None,
                None,
                None,
                Some("https://somewhere.com".to_string()),
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
";
        let bake_file = BakeFile::from_yaml(yaml).unwrap();

        dbg!(bake_file);
        // assert!(false);
    }
}
