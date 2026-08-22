use std::env;
#[derive(Debug, PartialEq)]
pub enum ParsedArgs {
    Version,
    Help,
    Update,
    ShowTasks,
    Serve(u16),            // (port)
    Command(String, bool), // (command_name, --non-interactive)
    Invalid,
    Nothing,
}

pub const DEFAULT_PORT: u16 = 8000;

pub fn get_args() -> ParsedArgs {
    get_args_with(env::args().collect())
}

fn get_args_with(args: Vec<String>) -> ParsedArgs {
    if let Some(arg1) = args.get(1) {
        match arg1.as_str() {
            "--show" | "--list" | "-l" => return ParsedArgs::ShowTasks,
            "--version" | "-v" => return ParsedArgs::Version,
            "--update" => return ParsedArgs::Update,
            "--help" => return ParsedArgs::Help,
            "serve" => return parse_serve(&args[2..]),
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

    ParsedArgs::Nothing
}

fn parse_serve(rest: &[String]) -> ParsedArgs {
    let mut port = DEFAULT_PORT;
    let mut i = 0;
    while i < rest.len() {
        let arg = &rest[i];
        if arg == "--port" {
            match rest.get(i + 1).and_then(|x| x.parse::<u16>().ok()) {
                Some(x) => port = x,
                None => return ParsedArgs::Invalid,
            }
            i += 2;
        } else if let Some(x) = arg.strip_prefix("--port=") {
            match x.parse::<u16>() {
                Ok(x) => port = x,
                Err(_) => return ParsedArgs::Invalid,
            }
            i += 1;
        } else {
            return ParsedArgs::Invalid;
        }
    }
    ParsedArgs::Serve(port)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        let mut v = vec!["bake".to_string()];
        v.extend(list.iter().map(|x| x.to_string()));
        v
    }

    #[test]
    fn serve_default_port() {
        assert_eq!(get_args_with(args(&["serve"])), ParsedArgs::Serve(8000));
    }

    #[test]
    fn serve_port() {
        assert_eq!(
            get_args_with(args(&["serve", "--port", "8080"])),
            ParsedArgs::Serve(8080)
        );
        assert_eq!(
            get_args_with(args(&["serve", "--port=1234"])),
            ParsedArgs::Serve(1234)
        );
    }

    #[test]
    fn serve_invalid() {
        assert_eq!(get_args_with(args(&["serve", "junk"])), ParsedArgs::Invalid);
        assert_eq!(
            get_args_with(args(&["serve", "--port", "not_a_number"])),
            ParsedArgs::Invalid
        );
    }
}
