use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub enum ParamValidator {
    Number, // Integer and float
    Integer,
    Variants(Vec<String>),
}

impl ParamValidator {
    pub fn validate(&self, value: &str) -> Result<(), String> {
        match self {
            ParamValidator::Integer => match value.parse::<i64>() {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_string()),
            },
            ParamValidator::Number => match value.parse::<f64>() {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_string()),
            },
            ParamValidator::Variants(variants) => variants
                .contains(&value.to_owned())
                .then_some(())
                .ok_or(format!("{} not in options{:?}", value, variants)),
        }
    }
}

impl TryFrom<&str> for ParamValidator {
    type Error = String;
    fn try_from(value: &str) -> Result<ParamValidator, String> {
        match value {
            "integer" | "int" => Ok(ParamValidator::Integer),
            "number" | "num" => Ok(ParamValidator::Number),
            x => {
                if let Some(x) = x.strip_prefix("variants") {
                    let x = x.trim();
                    if x.starts_with("(") && x.ends_with(")") {
                        let x = &x[1..x.len() - 1]; // remove "(" and ")"
                        let variants: Vec<String> = x
                            .split('|')
                            .map(|x| x.trim().to_string())
                            .filter(|x| !x.is_empty())
                            .collect();
                        Ok(ParamValidator::Variants(variants))
                    } else {
                        Err("invalid validation::variants".to_string())
                    }
                } else {
                    Err("invalid validation".to_string())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_enum_int_test() {
        assert_eq!(
            ParamValidator::try_from("integer"),
            Ok(ParamValidator::Integer)
        );
    }

    #[test]
    fn validation_enum_number_test() {
        assert_eq!(
            ParamValidator::try_from("integer"),
            Ok(ParamValidator::Integer)
        );
    }

    #[test]
    fn validation_enum_variants_test() {
        assert_eq!(
            ParamValidator::try_from("variants(a|b)"),
            Ok(ParamValidator::Variants(vec![
                "a".to_string(),
                "b".to_string()
            ]))
        );
    }
}
