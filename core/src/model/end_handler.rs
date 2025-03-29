use std::fmt::Display;

use super::function_call::FunctionCall;

#[derive(PartialEq, Debug)]
pub enum EndHandler {
    Restart,
    Retry(u32),
    FunctionCall(FunctionCall),
}

pub enum EndHandlerEvent {
    OnSuccess,
    OnError,
    OnEnd,
}

impl Display for EndHandlerEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EndHandlerEvent::OnSuccess => write!(f, "on_success"),
            EndHandlerEvent::OnError => write!(f, "on_error"),
            EndHandlerEvent::OnEnd => write!(f, "on_end"),
        }
    }
}

impl TryFrom<&str> for EndHandler {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let str = value.trim();
        if str.is_empty() {
            return Err("is empty".to_string());
        }

        if str.starts_with('@') {
            Ok(Self::FunctionCall(FunctionCall::try_from(str)?))
        } else if str == "restart" {
            Ok(Self::Restart)
        } else if str.starts_with("retry(") {
            let end_pos = match str.chars().position(|c| c == ')') {
                Some(x) => x,
                None => return Err(format!("invalid retry syntax: {}", str)),
            };
            let retries = &str[6..end_pos];
            let retries: Result<u32, _> = retries.parse();
            let retries = match retries {
                Ok(x) => x,
                Err(err) => return Err(err.to_string()),
            };
            Ok(Self::Retry(retries))
        } else {
            Err(format!("invalid end handler: {}", str))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn invalid_test() {
        assert!(EndHandler::try_from("brrrr").is_err());
    }

    #[test]
    fn restart_test() {
        assert_eq!(EndHandler::try_from("restart"), Ok(EndHandler::Restart));
    }

    #[test]
    fn function_call_test() {
        assert_eq!(
            EndHandler::try_from("@this.function_name"),
            Ok(EndHandler::FunctionCall(FunctionCall::new(
                "this".to_string(),
                "function_name".to_string(),
                HashMap::new()
            )))
        );
    }

    #[test]
    fn retry_test() {
        assert!(EndHandler::try_from("retry(5").is_err());
        assert!(EndHandler::try_from("retry()").is_err());
        assert!(EndHandler::try_from("retry(a)").is_err());
        assert_eq!(EndHandler::try_from("retry(5)"), Ok(EndHandler::Retry(5)));
    }
}
