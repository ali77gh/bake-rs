use std::fmt::Display;

pub enum EndHandler {
    OnSuccess,
    OnError,
    OnEnd,
}

impl Display for EndHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EndHandler::OnSuccess => write!(f, "on_success"),
            EndHandler::OnError => write!(f, "on_error"),
            EndHandler::OnEnd => write!(f, "on_end"),
        }
    }
}
