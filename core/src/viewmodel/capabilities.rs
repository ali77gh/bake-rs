pub trait Capabilities {
    /// returns None if file not bakefile exist
    fn read_file(&self, file_name: &str) -> Option<String>;

    /// std-out in Ok and std-err in Err
    fn execute(&self, command: &str) -> Result<String, String>;

    fn execute_all(&self, commands: &Vec<&str>) -> Result<Vec<String>, String> {
        let mut std_outs = vec![];
        for cmd in commands {
            match self.execute(&cmd) {
                Ok(std_out) => std_outs.push(std_out),
                Err(e) => return Err(e),
            }
        }
        Ok(std_outs)
    }

    fn open_link(&self, url: &str) -> Result<(), String>;

    fn std_out(&self, input: &str);
    fn std_in(&self) -> String;

    fn ask_user(&self, question: &str) -> String {
        self.std_out(format!("    {}? ", question).as_str());
        let answer = self.std_in();
        answer
    }

    fn ask_user_yes_no(&self, question: &str) -> bool {
        let answer = self
            .ask_user(format!("{} (yes|no)", question).as_str())
            .to_lowercase();
        let answer = answer.trim();
        answer == "yes" || answer == "y"
    }
}
