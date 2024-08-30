use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Param {
    name: String,
    validation: Option<ParamValidation>,
    value: Option<String>,
}

impl Param {
    pub fn new(name: String, validation: Option<ParamValidation>) -> Self {
        Self {
            name,
            validation,
            value: None,
        }
    }

    pub fn set_value(&mut self, value: String) -> Result<(), String> {
        if let Some(validation) = &self.validation {
            validation.validate(&value)?;
        }
        self.value = Some(value);
        Ok(())
    }

    pub fn get_value(&self) -> Option<String> {
        self.value.clone()
    }
}

// TODO specify min and max for numbers and string length

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ParamValidation {
    Number, // Integer and float
    Integer,
    Enum(Vec<String>),
}

impl ParamValidation {
    pub fn validate(&self, value: &str) -> Result<(), String> {
        match self {
            ParamValidation::Integer => match value.parse::<i64>() {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_string()),
            },
            ParamValidation::Number => match value.parse::<f64>() {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_string()),
            },
            ParamValidation::Enum(variants) => {
                for variant in variants {
                    if variant == value {
                        return Ok(());
                    }
                }
                Err("not in options".to_string())
            }
        }
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

    #[test]
    fn validation_none_test() {
        let mut param = Param::new("name".to_string(), None);
        assert_eq!(param.value, None);
        assert_eq!(param.set_value("1234".to_string()), Ok(()));
        assert_eq!(param.value, Some("1234".to_string()));
    }

    #[test]
    fn validation_integer_test() {
        let mut param = Param::new("name".to_string(), Some(ParamValidation::Integer));

        assert_eq!(param.value, None);
        assert_eq!(
            param.set_value("abc12".to_string()),
            Err("invalid digit found in string".to_string())
        );
        assert_eq!(param.value, None);
        assert_eq!(param.set_value("8765".to_string()), Ok(()));
        assert_eq!(param.value, Some("8765".to_string()));
    }

    #[test]
    fn validation_number_test() {
        let mut param = Param::new("name".to_string(), Some(ParamValidation::Number));

        assert_eq!(param.value, None);
        assert_eq!(
            param.set_value("abc".to_string()),
            Err("invalid float literal".to_string())
        );
        assert_eq!(param.value, None);

        // accepts integers
        assert_eq!(param.set_value("8765".to_string()), Ok(()));
        assert_eq!(param.value, Some("8765".to_string()));

        // accepts floats
        assert_eq!(param.set_value("3.14".to_string()), Ok(()));
        assert_eq!(param.value, Some("3.14".to_string()));
    }

    #[test]
    fn validation_enum() {
        let validation = ParamValidation::Enum(vec!["debug".to_string(), "release".to_string()]);
        let mut param = Param::new("name".to_string(), Some(validation));

        assert_eq!(param.value, None);
        assert_eq!(
            param.set_value("something_else".to_string()),
            Err("not in options".to_string())
        );
        assert_eq!(param.value, None);

        assert_eq!(param.set_value("debug".to_string()), Ok(()));
        assert_eq!(param.value, Some("debug".to_string()));
    }
}
