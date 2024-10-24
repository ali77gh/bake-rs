use serde::Deserialize;

use super::{dependency::Dependency, plugin::Plugin, task::Task};

#[derive(Debug, PartialEq, Deserialize)]
pub struct BakeFile {
    plugins: Option<Vec<Plugin>>,
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

    pub fn plugins(&self) -> &[Plugin] {
        if let Some(x) = &self.plugins {
            x
        } else {
            &[]
        }
    }

    pub fn dependencies(&self) -> &[Dependency] {
        if let Some(x) = &self.dependencies {
            x
        } else {
            &[]
        }
    }

    pub fn tasks(&self) -> &[Task] {
        if let Some(x) = &self.tasks {
            x
        } else {
            &[]
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

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
";
        let bake_file = BakeFile::from_yaml(yaml).unwrap();

        dbg!(bake_file);
        // assert!(false);
    }
}
