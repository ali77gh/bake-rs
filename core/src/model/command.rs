use serde::Deserialize;

use super::function_call::FunctionCall;

#[derive(Debug, PartialEq, Deserialize)]
pub enum Command {
    ShellCommand(String),
    FunctionCall(FunctionCall),
}

impl TryFrom<&str> for Command {
    type Error = String;
    fn try_from(str: &str) -> Result<Self, String> {
        let str = str.trim();
        if str.is_empty() {
            return Err("is empty".to_string());
        }
        if str.starts_with('@') {
            Ok(Self::FunctionCall(FunctionCall::try_from(str)?))
        } else {
            Ok(Self::ShellCommand(str.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn command_parser_shell_command_test() {
        let command = Command::try_from("cp from to");
        assert_eq!(command, Ok(Command::ShellCommand("cp from to".to_string())));
    }
    #[test]
    fn command_parser_function_call_test() {
        assert_eq!(
            Command::try_from("@fs.copy --from f --to t"),
            Ok(Command::FunctionCall(FunctionCall::new(
                "fs".to_string(),
                "copy".to_string(),
                HashMap::from([
                    ("from".to_string(), "f".to_string()),
                    ("to".to_string(), "t".to_string()),
                ])
            )))
        );
    }

    #[test]
    fn errors() {
        // empty
        assert_eq!(Command::try_from(""), Err("is empty".to_string()));

        // missing dot
        let command = Command::try_from("@fscopy from to");
        assert_eq!(command, Err("syntax error".to_string()));

        // missing namespace
        let command = Command::try_from("@.copy from to");
        assert_eq!(
            command,
            Err("syntax error(namespace is missing)".to_string())
        );

        // missing function name
        let command = Command::try_from("@fs. from to");
        assert_eq!(
            command,
            Err("syntax error(function name is missing)".to_string())
        );

        // missing params (ok)
        let command = Command::try_from("@fs.copy");
        assert_eq!(
            command,
            Ok(Command::FunctionCall(FunctionCall::new(
                "fs".to_string(),
                "copy".to_string(),
                HashMap::new()
            )))
        );
    }
}
