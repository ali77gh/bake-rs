pub struct Message {
    message_type: MessageType,
    content: String,
}

impl Message {
    fn new(message_type: MessageType, content: String) -> Self {
        Self {
            message_type,
            content,
        }
    }

    pub fn error(str: impl ToString) -> Self {
        Self::new(MessageType::Error, str.to_string())
    }
    pub fn bake_state(str: impl ToString) -> Self {
        Self::new(MessageType::BakeState, str.to_string())
    }
    pub fn warning(str: impl ToString) -> Self {
        Self::new(MessageType::Warning, str.to_string())
    }
    pub fn normal(str: impl ToString) -> Self {
        Self::new(MessageType::Normal, str.to_string())
    }
    pub fn question(str: impl ToString) -> Self {
        Self::new(MessageType::Question, str.to_string())
    }

    pub fn message_type(&self) -> &MessageType {
        &self.message_type
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

pub enum MessageType {
    Error,
    BakeState,
    Warning,
    Normal,
    Question,
}
