use serde::Deserialize;

use super::param_validator::ParamValidator;

/// yaml model for task params
#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Param {
    name: String,
    validator: Option<String>,
    default: Option<String>,
}

impl Param {
    pub fn new(name: String, validation: Option<String>) -> Self {
        Self {
            name,
            validator: validation,
            default: None,
        }
    }

    pub fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn validator(&self) -> Option<Result<ParamValidator, String>> {
        self.validator
            .as_ref()
            .map(|x| ParamValidator::try_from(x.as_str()))
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
