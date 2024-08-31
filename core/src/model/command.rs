use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Command {
    ShellCommand(String),
    FunctionCall(FunctionCall),
}

impl Command {
    pub fn from_str(str: impl AsRef<str>) -> Result<Self, String> {
        let str = str.as_ref().trim();
        if str.is_empty() {
            return Err("is empty".to_string());
        }
        if str.starts_with('@') {
            return Ok(Self::FunctionCall(FunctionCall::from_str(str)?));
        } else {
            return Ok(Self::ShellCommand(str.to_string()));
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FunctionCall {
    namespace: String,
    function: String,
    params: Vec<String>,
}

impl FunctionCall {
    pub fn new(namespace: String, function: String, params: Vec<String>) -> Self {
        Self {
            namespace,
            function,
            params,
        }
    }

    pub fn from_str(str: impl AsRef<str>) -> Result<Self, String> {
        let str = str.as_ref().trim();

        // Safety input string at
        let str = &str[1..]; // pop '@' from begin
        let binding = str.split(' ').collect::<Vec<&str>>();
        let function_id = binding.first();
        let function_id = function_id.ok_or("syntax error".to_string())?;

        let sp = function_id.split('.').collect::<Vec<&str>>();
        let namespace = sp.get(0).ok_or("syntax error".to_string())?;
        let function_name = sp.get(1).ok_or("syntax error".to_string())?;

        let params = str
            .split(' ')
            .skip(1)
            .map(String::from)
            .collect::<Vec<String>>();

        if namespace.is_empty() {
            return Err("syntax error(namespace is missing)".to_string());
        }
        if function_name.is_empty() {
            return Err("syntax error(function name is missing)".to_string());
        }

        Ok(Self::new(
            namespace.to_string(),
            function_name.to_string(),
            params,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_parser_shell_command_test() {
        let command = Command::from_str("cp from to");
        assert_eq!(command, Ok(Command::ShellCommand("cp from to".to_string())));
    }
    #[test]
    fn command_parser_function_call_test() {
        let command = Command::from_str("@fs.copy from to");
        assert_eq!(
            command,
            Ok(Command::FunctionCall(FunctionCall::new(
                "fs".to_string(),
                "copy".to_string(),
                vec!["from".to_string(), "to".to_string()]
            )))
        );
    }

    #[test]
    fn errors() {
        // empty
        assert_eq!(Command::from_str(""), Err("is empty".to_string()));

        // missing dot
        let command = Command::from_str("@fscopy from to");
        assert_eq!(command, Err("syntax error".to_string()));

        // missing namespace
        let command = Command::from_str("@.copy from to");
        assert_eq!(
            command,
            Err("syntax error(namespace is missing)".to_string())
        );

        // missing function name
        let command = Command::from_str("@fs. from to");
        assert_eq!(
            command,
            Err("syntax error(function name is missing)".to_string())
        );

        // missing empty params
        let command = Command::from_str("@fs.copy");
        assert_eq!(
            command,
            Ok(Command::FunctionCall(FunctionCall {
                namespace: "fs".to_string(),
                function: "copy".to_string(),
                params: vec![]
            }))
        );
    }
}
