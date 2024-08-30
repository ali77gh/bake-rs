use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Command {
    ShellCommand(String),
    FunctionCall(FunctionCall),
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FunctionCall {
    namespace: String,
    function: String,
    params: HashMap<String, String>,
}
