mod show_tasks;

use core::viewmodel::capabilities::Capabilities;
use core::viewmodel::BakeViewModel;
use std::process::Command;
use std::rc::Rc;

fn main() {
    let bake = BakeViewModel::new(Rc::new(CLICapabilities)).expect("bakefile not found");

    show_tasks::show_tasks(bake.tasks());
}

struct CLICapabilities;
impl Capabilities for CLICapabilities {
    fn read_file(&self, file_name: &str) -> Option<String> {
        std::fs::read_to_string(file_name).ok()
    }

    fn execute(&self, command: &str) -> Result<String, String> {
        let result = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", command]).output()
        } else {
            Command::new("sh").arg("-c").arg(command).output()
        };

        match result {
            Ok(x) => match std::str::from_utf8(&x.stdout) {
                Ok(x) => Ok(x.to_string()),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }

    fn open_link(&self, url: &str) -> Result<(), String> {
        let url = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("http://{}", url)
        } else {
            url.to_owned()
        };

        match webbrowser::open(&url) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}
