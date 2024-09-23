#![allow(unused_variables)]

pub fn platform_specific<'a, T>(
    common: Option<&'a T>,
    linux: Option<&'a T>,
    windows: Option<&'a T>,
    macos: Option<&'a T>,
) -> Option<&'a T> {
    #[cfg(target_os = "linux")]
    if let Some(s) = linux {
        return Some(s);
    }
    #[cfg(target_os = "windows")]
    if let Some(s) = windows {
        return Some(s);
    }
    #[cfg(target_os = "macos")]
    if let Some(s) = macos {
        return Some(s);
    }
    common
}

pub fn get_platform_name() -> &'static str {
    #[cfg(target_os = "linux")]
    return "Linux";
    #[cfg(target_os = "windows")]
    return "Windows";
    #[cfg(target_os = "macos")]
    return "MacOS";
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_supported_test() {
        let macos = vec!["cmd1".to_string(), "cmd2".to_string()];
        let ps = platform_specific(None, None, None, Some(&macos));
        assert_eq!(ps, None);
    }

    #[test]
    fn main_test() {
        let common = vec!["cmd1".to_string(), "cmd2".to_string()];
        let ps = platform_specific(Some(&common), None, None, None);
        assert_eq!(ps.unwrap().len(), 2)
    }

    #[test]
    fn linux_only_test() {
        let linux = vec!["cmd1".to_string(), "cmd2".to_string()];
        let ps = platform_specific(None, Some(&linux), None, None);
        assert_eq!(ps.unwrap().len(), 2)
    }
}
