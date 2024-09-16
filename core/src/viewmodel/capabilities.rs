use super::message::Message;

pub trait Capabilities {
    /// returns None if file not bakefile exist
    fn read_file(&self, file_name: &str) -> Option<String>;

    /// std-out in Ok and std-err in Err
    fn execute_silent(&self, command: &str) -> Result<String, String>;

    fn execute_and_print(&self, command: &str) -> Result<(), String> {
        self.message(Message::new(
            super::message::MessageType::Normal,
            self.execute_silent(command)?,
        ));
        Ok(())
    }

    fn execute_and_print_all(&self, commands: &[&str]) -> Result<(), String> {
        for cmd in commands {
            self.message(Message::new(
                super::message::MessageType::BakeState,
                format!("command '{}' is running...\n", cmd),
            ));
            self.execute_and_print(cmd)?;
        }
        Ok(())
    }

    fn execute_silent_all(&self, commands: &[&str]) -> Result<(), String> {
        for cmd in commands {
            self.execute_silent(cmd)?;
        }
        Ok(())
    }

    fn open_link(&self, url: &str) -> Result<(), String>;

    fn message(&self, input: Message);
    fn input(&self) -> String;

    fn ask_user(&self, question: &str) -> String {
        self.message(Message::new(
            super::message::MessageType::Question,
            question.to_string(),
        ));
        self.input()
    }

    fn ask_user_yes_no(&self, question: &str) -> bool {
        let answer = self
            .ask_user(format!("{} (yes|no)", question).as_str())
            .to_lowercase();
        let answer = answer.trim();
        answer == "yes" || answer == "y"
    }
}
