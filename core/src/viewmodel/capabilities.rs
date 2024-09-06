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
}
