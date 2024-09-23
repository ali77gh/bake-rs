use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
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
