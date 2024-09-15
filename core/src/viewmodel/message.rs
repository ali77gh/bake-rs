pub struct Message {
    message_type: MessageType,
    content: String,
}

impl Message {
    pub fn new(message_type: MessageType, content: String) -> Self {
        Self {
            message_type,
            content,
        }
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
