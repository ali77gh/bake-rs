mod capabilities;

use capabilities::Capabilities;

use crate::model::bake_file::BakeFile;

const BAKE_FILE_NAME: &str = "bakefile.yaml";

pub struct BakeViewModel {
    bake_file: BakeFile,
    caps: Box<dyn Capabilities>,
}

impl BakeViewModel {
    /// returns None if file not bakefile exist
    pub fn new(caps: Box<dyn Capabilities>) -> Result<Self, String> {
        if let Some(content) = caps.read_file(BAKE_FILE_NAME) {
            Ok(Self {
                bake_file: BakeFile::from_yaml(&content)?,
                caps,
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
    }

    #[test]
    fn test_name() {
        let cap = TestCap;
        let r = BakeViewModel::new(Box::new(cap));
        assert_eq!(
            r.unwrap().bake_file().tasks().get(0).unwrap().name(),
            "clean".to_string()
        );
    }
}
