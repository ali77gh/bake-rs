use std::env;
pub enum ParsedArgs {
    Version,
    Help,
    Update,
    ShowTasks,
    Command(String, bool), // (command_name, --non-interactive)
    Invalid,
}

pub fn get_args() -> ParsedArgs {
    let args: Vec<String> = env::args().collect();
    if let Some(arg1) = args.get(1) {
        match arg1.as_str() {
            "--show" | "--list" | "-l" => return ParsedArgs::ShowTasks,
            "--version" | "-v" => return ParsedArgs::Version,
            "--update" => return ParsedArgs::Update,
            "--help" => return ParsedArgs::Help,
            _ => {
                if !arg1.starts_with("--") {
                    match args.get(2) {
                        Some(arg2) => {
                            return ParsedArgs::Command(
                                arg1.clone(),
                                arg2 == "--non-interactive" || arg2 == "-ni",
                            );
                        }
                        None => return ParsedArgs::Command(arg1.clone(), false),
                    }
                } else {
                    return ParsedArgs::Invalid;
                }
            }
        }
    }

    ParsedArgs::Invalid
}
