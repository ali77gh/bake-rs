use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
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

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn function(&self) -> &str {
        &self.function
    }

    pub fn params(&self) -> &[String] {
        &self.params
    }
}

impl TryFrom<&str> for FunctionCall {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, String> {
        let str = value.trim();

        // Safety input string at
        let str = &str[1..]; // pop '@' from begin
        let binding = str.split(' ').collect::<Vec<&str>>();
        let function_id = binding.first();
        let function_id = function_id.ok_or("syntax error".to_string())?;

        let sp = function_id.split('.').collect::<Vec<&str>>();
        let namespace = sp.first().ok_or("syntax error".to_string())?;
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
        let command = Command::try_from("cp from to");
        assert_eq!(command, Ok(Command::ShellCommand("cp from to".to_string())));
    }
    #[test]
    fn command_parser_function_call_test() {
        let command = Command::try_from("@fs.copy from to");
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

        // missing empty params
        let command = Command::try_from("@fs.copy");
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
