use std::env;
pub enum ParsedArgs {
    Version,
    Help,
    Update,
    ShowTasks,
    Command(String),
    Invalid,
}

pub fn get_args() -> ParsedArgs {
    let args: Vec<String> = env::args().collect();
    if let Some(x) = args.get(1) {
        match x.as_str() {
            "--show" | "--list" | "-l" => return ParsedArgs::ShowTasks,
            "--version" | "-v" => return ParsedArgs::Version,
            "--update" => return ParsedArgs::Update,
            "--help" => return ParsedArgs::Help,
            _ => {
                if !x.starts_with("--") {
                    return ParsedArgs::Command(x.clone());
                } else {
                    return ParsedArgs::Invalid;
                }
            }
        }
    }

    ParsedArgs::Invalid
}
