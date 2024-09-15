use core::viewmodel::capabilities::Capabilities;
use core::viewmodel::message::Message;
use std::io::Write;
use std::process::Command;

pub struct CLICapabilities;
impl Capabilities for CLICapabilities {
    fn read_file(&self, file_name: &str) -> Option<String> {
        std::fs::read_to_string(file_name).ok()
    }

    fn execute_silent(&self, command: &str) -> Result<String, String> {
        let result = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", command]).output()
        } else {
            Command::new("sh").arg("-c").arg(command).output()
        };

        match result {
            Ok(x) => {
                if x.status.success() {
                    match std::str::from_utf8(&x.stdout) {
                        Ok(x) => Ok(x.to_string()),
                        Err(e) => Err(e.to_string()),
                    }
                } else {
                    match std::str::from_utf8(&x.stderr) {
                        Ok(x) => Err(x.to_string()),
                        Err(e) => Err(e.to_string()),
                    }
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    fn open_link(&self, url: &str) -> Result<(), String> {
        match webbrowser::open(&url) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn message(&self, input: Message) {
        print!("{}", input.content());
        std::io::stdout().flush().unwrap();
    }

    fn input(&self) -> String {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        buffer
    }
}