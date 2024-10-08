use serde::Deserialize;

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
