use core::viewmodel::{capabilities::Capabilities, message::Message, BakeViewModel};
use std::{process::exit, rc::Rc};

use crate::{capabilities::CLICapabilities, show_tasks::show_tasks};

pub fn start_repl() {
    let caps = Rc::new(CLICapabilities::default());

    let bake_caps = Rc::clone(&caps);
    let bake = match BakeViewModel::new(bake_caps) {
        Ok(x) => x,
        Err(e) => {
            println!("{}", e);
            exit(1); // cleaner way to panic
        }
    };

    loop {
        show_tasks(bake.tasks());
        let task_name = match caps.ask_user("which task do you want to run (enter name or index)") {
            Some(x) => x,
            None => {
                println!("Bake: bye bye!");
                exit(1);
            }
        };

        let task_name = task_name.trim();
        if let Err(e) = bake.run_task(task_name) {
            CLICapabilities::default().message(Message::error(format!("{e}\n")));
            CLICapabilities::default().message(Message::error(format!(
                "Task '{task_name}' failed to run\n"
            )));
        }
        println!("\n──────────Bake-Loop──────────\n");
    }
}
