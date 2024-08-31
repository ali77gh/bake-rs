pub fn platform_specific<'a>(
    common: Option<&'a Vec<String>>,
    linux: Option<&'a Vec<String>>,
    windows: Option<&'a Vec<String>>,
    macos: Option<&'a Vec<String>>,
) -> Option<&'a Vec<String>> {
    #[cfg(target_os = "linux")]
    if let Some(s) = linux {
        if !s.is_empty() {
            return Some(s);
        }
    }
    #[cfg(target_os = "windows")]
    if let Some(s) = windows {
        if !s.is_empty() {
            return Some(s);
        }
    }
    #[cfg(target_os = "macos")]
    if let Some(s) = macos {
        if !s.is_empty() {
            return Some(s);
        }
    }
    return common;
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