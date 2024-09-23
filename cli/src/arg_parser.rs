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
        if x == "--show" {
            return ParsedArgs::ShowTasks;
        }

        if x == "--version" || x == "-v" {
            return ParsedArgs::Version;
        }

        if x == "--update" {
            return ParsedArgs::Update;
        }

        if x == "--help" {
            return ParsedArgs::Help;
        }

        if !x.starts_with("--") {
            return ParsedArgs::Command(x.clone());
        }
    }

    ParsedArgs::Invalid
}
