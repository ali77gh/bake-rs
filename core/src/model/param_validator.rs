use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)] // everything else in this yaml is lower case
pub enum ParamValidator {
    number, // Integer and float
    integer,
    variants(Vec<String>),
}

impl ParamValidator {
    pub fn validate(&self, value: &str) -> Result<(), String> {
        match self {
            ParamValidator::integer => match value.parse::<i64>() {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_string()),
            },
            ParamValidator::number => match value.parse::<f64>() {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_string()),
            },
            ParamValidator::variants(variants) => {
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
    fn validation_enum_serde_test() {
        let str = serde_yaml::to_string(&ParamValidator::variants(vec![
            "debug".to_string(),
            "release".to_string(),
        ]))
        .unwrap();

        assert_eq!(str, "!variants\n- debug\n- release\n");
    }
}
