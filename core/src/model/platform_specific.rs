use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct PlatformSpecific {
    main: Option<Vec<String>>,
    linux: Option<Vec<String>>,
    windows: Option<Vec<String>>,
    macos: Option<Vec<String>>,
}

impl PlatformSpecific {
    pub fn new(
        main: Option<Vec<String>>,
        linux: Option<Vec<String>>,
        windows: Option<Vec<String>>,
        macos: Option<Vec<String>>,
    ) -> Self {
        Self {
            main,
            linux,
            windows,
            macos,
        }
    }

    pub fn get_list(self) -> Result<Vec<String>, String> {
        #[cfg(target_os = "linux")]
        return self
            .linux
            .or(self.main)
            .ok_or("not supported on linux".to_string());

        #[cfg(target_os = "windows")]
        return self
            .windows
            .or(self.main)
            .ok_or("not supported on windows".to_string());

        #[cfg(target_os = "macos")]
        return self
            .mac_os
            .or(self.main)
            .ok_or("not supported on macOS".to_string());
    }

    pub fn get_platform_support_text(&self) -> Option<String> {
        match (&self.linux, &self.windows, &self.macos, &self.main) {
            (Some(_), None, None, None) => Some("linux only".to_string()),
            (None, Some(_), None, None) => Some("windows only".to_string()),
            (None, None, Some(_), None) => Some("macOS only".to_string()),
            _ => None,
        }
    }
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn not_supported_test() {
    //     let ps = PlatformSpecific::<String>::new(
    //         None,
    //         None,
    //         None,
    //         Some(vec!["cmd1".to_string(), "cmd2".to_string()]), // macOS only
    //     );
    //     assert_eq!(ps.get_list(), Err("not supported on linux".to_string()))
    // }

    // #[test]
    // fn main_test() {
    //     let ps = PlatformSpecific::<String>::new(
    //         Some(vec!["cmd1".to_string(), "cmd2".to_string()]), // main only
    //         None,
    //         None,
    //         None,
    //     );
    //     assert_eq!(ps.get_list().unwrap().len(), 2)
    // }

    // #[test]
    // fn linux_only_test() {
    //     let ps = PlatformSpecific::<String>::new(
    //         None,
    //         Some(vec!["cmd1".to_string(), "cmd2".to_string()]), // linux only
    //         None,
    //         None,
    //     );
    //     assert_eq!(ps.get_list().unwrap().len(), 2)
    // }

    // #[test]
    // fn platform_only_test() {
    //     let ps = PlatformSpecific::<String>::new(
    //         None,
    //         Some(vec!["cmd1".to_string(), "cmd2".to_string()]),
    //         None,
    //         None,
    //     );
    //     assert_eq!(
    //         ps.get_platform_support_text(),
    //         Some("linux only".to_string())
    //     );

    //     let ps = PlatformSpecific::<String>::new(
    //         Some(vec!["cmd1".to_string(), "cmd2".to_string()]),
    //         None,
    //         None,
    //         None,
    //     );
    //     assert_eq!(ps.get_platform_support_text(), None);

    //     let ps = PlatformSpecific::<String>::new(None, None, None, None);
    //     assert_eq!(ps.get_platform_support_text(), None);

    //     let ps = PlatformSpecific::<String>::new(
    //         None,
    //         Some(vec!["cmd1".to_string(), "cmd2".to_string()]),
    //         Some(vec!["cmd1".to_string(), "cmd2".to_string()]),
    //         None,
    //     );
    //     assert_eq!(ps.get_platform_support_text(), None);
    // }
}
