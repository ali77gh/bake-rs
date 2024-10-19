use core::viewmodel::capabilities::Capabilities;
use core::viewmodel::message::{Message, MessageType};
use std::io::Write;
use std::process::{Command, Stdio};

pub struct CLICapabilities {
    pub non_interactive: bool,
}

impl Default for CLICapabilities {
    fn default() -> Self {
        Self {
            non_interactive: false,
        }
    }
}
impl Capabilities for CLICapabilities {
    fn read_file(&self, file_name: &str) -> Option<String> {
        std::fs::read_to_string(file_name).ok()
    }

    fn execute(&self, command: &str) -> bool {
        let result = Command::new(SHELL)
            .arg(SWITCH)
            .arg(command)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap()
            .wait();

        match result {
            Ok(x) => x.success(),
            Err(_) => false,
        }
    }

    fn open_link(&self, url: &str) -> Result<(), String> {
        match webbrowser::open(url) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn message(&self, input: Message) {
        use colored::Colorize;

        // prevent printing message in non-interactive mode
        if self.non_interactive && *input.message_type() == MessageType::Question {
            return;
        }

        match input.message_type() {
            core::viewmodel::message::MessageType::Error => {
                print!(
                    " {}: {}",
                    " ❌ Error ".on_red().bold(),
                    input.content().red()
                )
            }
            core::viewmodel::message::MessageType::BakeState => {
                print!(
                    " {}: {}",
                    " ▶ Bake ".on_bright_yellow().black(),
                    input.content().bright_yellow()
                )
            }
            core::viewmodel::message::MessageType::Warning => {
                print!(
                    " {}: {}",
                    " ⚠ Warning ".on_yellow().bold(),
                    input.content().yellow()
                )
            }
            core::viewmodel::message::MessageType::Normal => print!("{}", input.content()),
            core::viewmodel::message::MessageType::Question => {
                print!(
                    " {}: {}? ",
                    " 🯄 Question ".on_bright_yellow().black(),
                    input.content().bright_yellow()
                )
            }
        }

        std::io::stdout().flush().unwrap();
    }

    fn input(&self) -> Option<String> {
        if self.non_interactive {
            None
        } else {
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer).unwrap();
            Some(buffer)
        }
    }
}

#[cfg(target_os = "windows")]
const SHELL: &str = "cmd";
#[cfg(target_os = "windows")]
const SWITCH: &str = "/C";
#[cfg(not(target_os = "windows"))]
const SHELL: &str = "sh";
#[cfg(not(target_os = "windows"))]
const SWITCH: &str = "-c";
