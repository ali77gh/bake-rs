use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Param {
    name: String,
    validator: Option<ParamValidation>,
    value: Option<String>,
}

impl Param {
    pub fn new(name: String, validation: Option<ParamValidation>) -> Self {
        Self {
            name,
            validator: validation,
            value: None,
        }
    }

    pub fn set_value(&mut self, value: String) -> Result<(), String> {
        if let Some(validation) = &self.validator {
            validation.validate(&value)?;
        }
        self.value = Some(value);
        Ok(())
    }

    pub fn get_value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn validator(&self) -> Option<&ParamValidation> {
        self.validator.as_ref()
    }
}

// TODO specify min and max for numbers and string length

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)] // everything else in this yaml is lower case
pub enum ParamValidation {
    number, // Integer and float
    integer,
    variants(Vec<String>),
}

impl ParamValidation {
    pub fn validate(&self, value: &str) -> Result<(), String> {
        match self {
            ParamValidation::integer => match value.parse::<i64>() {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_string()),
            },
            ParamValidation::number => match value.parse::<f64>() {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_string()),
            },
            ParamValidation::variants(variants) => {
                for variant in variants {
                    if variant == value {
                        return Ok(());
                    }
                }
                Err(format!("{} not in options{:?}", value, variants))
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
        let mut param = Param::new("name".to_string(), Some(ParamValidation::integer));

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
        let mut param = Param::new("name".to_string(), Some(ParamValidation::number));

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
        let validation =
            ParamValidation::variants(vec!["debug".to_string(), "release".to_string()]);
        let mut param = Param::new("name".to_string(), Some(validation));

        assert_eq!(param.value, None);
        assert_eq!(
            param.set_value("something_else".to_string()),
            Err("something_else not in options[\"debug\", \"release\"]".to_string())
        );
        assert_eq!(param.value, None);

        assert_eq!(param.set_value("debug".to_string()), Ok(()));
        assert_eq!(param.value, Some("debug".to_string()));
    }

    #[test]
    fn validation_enum_serde_test() {
        let str = serde_yaml::to_string(&ParamValidation::variants(vec![
            "debug".to_string(),
            "release".to_string(),
        ]))
        .unwrap();

        assert_eq!(str, "!variants\n- debug\n- release\n");
    }
}
