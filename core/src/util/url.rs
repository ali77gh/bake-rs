use crate::util::platform_specific::get_platform_name;

/// Generates google search URL
pub fn generate_installation_link(name: &str) -> String {
    standard_link(&format!(
        "https://google.com/search?q=how+to+install+{}+on+{}",
        name,
        get_platform_name()
    ))
}

/// ands https if needed
pub fn standard_link(url: &str) -> String {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        format!("http://{}", url)
    } else {
        url.to_owned()
    }
}
