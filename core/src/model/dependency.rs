use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    name: String,
    dependencies: Option<Vec<String>>,

    check: Option<Vec<String>>,
    check_linux: Option<Vec<String>>,
    check_windows: Option<Vec<String>>,
    check_macos: Option<Vec<String>>,

    link: Option<Vec<String>>,
    link_linux: Option<Vec<String>>,
    link_windows: Option<Vec<String>>,
    link_macos: Option<Vec<String>>,

    command: Option<Vec<String>>,
    command_linux: Option<Vec<String>>,
    command_windows: Option<Vec<String>>,
    command_macos: Option<Vec<String>>,
}

impl Dependency {
    pub fn new(
        name: String,
        dependencies: Option<Vec<String>>,
        check: Option<Vec<String>>,
        check_linux: Option<Vec<String>>,
        check_windows: Option<Vec<String>>,
        check_macos: Option<Vec<String>>,
        link: Option<Vec<String>>,
        link_linux: Option<Vec<String>>,
        link_windows: Option<Vec<String>>,
        link_macos: Option<Vec<String>>,
        command: Option<Vec<String>>,
        command_linux: Option<Vec<String>>,
        command_windows: Option<Vec<String>>,
        command_macos: Option<Vec<String>>,
    ) -> Self {
        Self {
            name,
            dependencies,
            check,
            check_linux,
            check_windows,
            check_macos,
            link,
            link_linux,
            link_windows,
            link_macos,
            command,
            command_linux,
            command_windows,
            command_macos,
        }
    }
}
