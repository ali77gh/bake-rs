use super::message::Message;

/// This trait makes view_model pure and side_effect independent
/// it will implemented by CLI and GUI crate differently
/// we put all side effects in here
pub trait Capabilities {
    /// returns [None] if file not exist
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

    /// open link in user browser or just show user the link
    fn open_link(&self, url: &str);

    /// message from bake to user
    /// also consider to user [Message::message_type] to use colors and more
    fn message(&self, input: Message);

    /// take input from user
    /// return [None] if you cant or don't want to (example: --non-interactive)
    fn input(&self) -> Option<String>;

    /// show user question and takes input from user
    /// override this if it's not simple as calling message() and input()
    fn ask_user(&self, question: &str) -> Option<String> {
        self.message(Message::question(question));
        self.input()
    }

    /// same as [ask_user] but it's a yes/no question
    fn ask_user_yes_no(&self, question: &str) -> Option<bool> {
        let answer = self
            .ask_user(format!("{} (yes|no)", question).as_str())?
            .to_lowercase();
        let answer = answer.trim();
        Some(answer == "yes" || answer == "y")
    }

    /// standard env set implementation
    fn set_env(&self, name: &str, value: &str);

    /// standard env get implementation
    fn get_env(&self, name: &str) -> Option<String>;

    fn remove_env(&self, name: &str);
}
