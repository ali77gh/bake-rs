use serde::{Deserialize, Serialize};

use super::param_validator::ParamValidator;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Param {
    name: String,
    validator: Option<ParamValidator>,
    value: Option<String>,
}

impl Param {
    pub fn new(name: String, validation: Option<ParamValidator>) -> Self {
        Self {
            name,
            validator: validation,
            value: None,
        }
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn validator(&self) -> Option<&ParamValidator> {
        self.validator.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_name_test() {
        let param = Param::new("name".to_string(), None);
        assert_eq!(param.name, "name".to_string());
    }
}
