use std::collections::HashMap;

pub enum Command {
    ShellCommand(String),
    FunctionCall(FunctionCall),
}

pub struct FunctionCall {
    namespace: String,
    function: String,
    params: HashMap<String, String>,
}
