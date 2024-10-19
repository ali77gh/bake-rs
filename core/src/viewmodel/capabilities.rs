use super::message::Message;

pub trait Capabilities {
    /// returns None if file not bakefile exist
    fn read_file(&self, file_name: &str) -> Option<String>;

    /// returns true if non zero code
    /// interact with user however you want
    /// Suggestion:
    ///     stdout to [Message::normal]
    ///     stderr to [Message::error]
    ///     stdin using [Capabilities::ask_user]
    fn execute(&self, command: &str) -> bool;

    /// doing [Capabilities::execute] in loop and exit on false
    fn execute_all(&self, commands: &[&str]) -> bool {
        for cmd in commands {
            if !self.execute(cmd) {
                return false;
            }
        }
        true
    }

    fn open_link(&self, url: &str) -> Result<(), String>;

    fn message(&self, input: Message);
    fn input(&self) -> Option<String>;

    fn ask_user(&self, question: &str) -> Option<String> {
        self.message(Message::question(question));
        self.input()
    }

    fn ask_user_yes_no(&self, question: &str) -> Option<bool> {
        let answer = self
            .ask_user(format!("{} (yes|no)", question).as_str())?
            .to_lowercase();
        let answer = answer.trim();
        Some(answer == "yes" || answer == "y")
    }
}
