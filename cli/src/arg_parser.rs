use std::env;
pub enum ParsedArgs {
    ShowTasks,
    Invalid,
}

pub fn get_args() -> ParsedArgs {
    let args: Vec<String> = env::args().collect();
    if let Some(x) = args.get(1) {
        if x == "--show" {
            return ParsedArgs::ShowTasks;
        }
    }

    return ParsedArgs::Invalid;
}
