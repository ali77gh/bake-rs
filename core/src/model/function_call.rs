use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct FunctionCall {
    namespace: String,
    function: String,
    params: HashMap<String, String>,
}

impl FunctionCall {
    pub fn new(namespace: String, function: String, params: HashMap<String, String>) -> Self {
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

    pub fn params(&self) -> &HashMap<String, String> {
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
        let function_id =
            function_id.ok_or("syntax error(@namespace.functionName not found)".to_string())?;

        let sp = function_id.split('.').collect::<Vec<&str>>();
        let (namespace, function_name) = match sp.len() {
            // Safety: len is 1 so first always exist
            1 => (&"this", sp.first().unwrap()),
            // Safety: len is 2 so first and second always exist
            2 => (sp.first().unwrap(), sp.get(1).unwrap()),
            _ => return Err("syntax error( two dots found in @namespace.functionName)".to_string()),
        };

        if namespace.is_empty() {
            return Err("syntax error(namespace is missing)".to_string());
        }
        if function_name.is_empty() {
            return Err("syntax error(function name is missing)".to_string());
        }

        let mut params = HashMap::<String, String>::new();
        let sp = str.split(' ').skip(1).collect::<Vec<&str>>(); // skip namespace and function name
        if !sp.is_empty() {
            if sp.len() % 2 != 0 {
                return Err("invalid param passing. help: @namespace.task --param1 value1 --param2 value2 ...".to_owned());
            }
            for i in 0..sp.len() - 1 {
                if i % 2 == 0 {
                    let key = sp.get(i).unwrap().to_string();
                    let value = sp.get(i + 1).unwrap().to_string();

                    if !key.starts_with("--") {
                        return Err(format!(
                            "invalid param name '{}' (param name should start with '--' )",
                            key
                        ));
                    }
                    let key = key[2..].to_string(); // pop -- from key
                    params.insert(key, value);
                }
            }
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
    fn valid() {
        assert_eq!(
            FunctionCall::try_from("@this.function_name").unwrap(),
            FunctionCall::new(
                "this".to_string(),
                "function_name".to_string(),
                HashMap::new()
            )
        );

        assert_eq!(
            FunctionCall::try_from("@function_name").unwrap(),
            FunctionCall::new(
                "this".to_string(),
                "function_name".to_string(),
                HashMap::new()
            )
        );

        assert_eq!(
            FunctionCall::try_from("@this.function_name --param1 a --param2 b").unwrap(),
            FunctionCall::new(
                "this".to_string(),
                "function_name".to_string(),
                HashMap::from([
                    ("param1".to_string(), "a".to_string()),
                    ("param2".to_string(), "b".to_string())
                ])
            )
        );

        assert_eq!(
            FunctionCall::try_from("@this.function_name --param1 --param2").unwrap(),
            FunctionCall::new(
                "this".to_string(),
                "function_name".to_string(),
                HashMap::from([("param1".to_string(), "--param2".to_string()),])
            )
        );
    }

    #[test]
    fn invalid() {
        assert_eq!(
            FunctionCall::try_from("@this.function_name a"),
            Err(
                "invalid param passing. help: @namespace.task --param1 value1 --param2 value2 ..."
                    .to_string()
            )
        );

        assert_eq!(
            FunctionCall::try_from("@this.function_name a b"),
            Err("invalid param name 'a' (param name should start with '--' )".to_string())
        );

        assert_eq!(
            FunctionCall::try_from("@this.function_name --param1 a --param2"),
            Err(
                "invalid param passing. help: @namespace.task --param1 value1 --param2 value2 ..."
                    .to_string()
            )
        );
    }
}
