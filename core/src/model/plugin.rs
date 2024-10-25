use serde::Deserialize;

/// yaml model for [Plugin]
/// name used in function call syntax as namespace see here: [crate::model::function_call::FunctionCall::namespace]
#[derive(Debug, PartialEq, Deserialize)]
pub struct Plugin {
    name: String,
    path: String,
}

impl Plugin {
    pub fn new(name: String, path: String) -> Self {
        Self { name, path }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}
