use core::viewmodel::{capabilities::Capabilities, message::Message, BakeViewModel};
use std::{process::exit, rc::Rc};

use crate::{capabilities::CLICapabilities, show_tasks::show_tasks};

pub fn start_repl() {
    let caps = Rc::new(CLICapabilities::default());

    loop {
        let bake = new_bake(&caps);
        show_tasks(bake.tasks());
        let task_name = match caps.ask_user("which task do you want to run (enter name or index)") {
            Some(x) => x,
            None => {
                println!("Bake: bye bye!");
                exit(1);
            }
        };

        // read the bakefile again to get new changes happened during ask_user
        let bake = new_bake(&caps);

        let task_name = task_name.trim();
        if let Err(e) = bake.run_task(task_name) {
            caps.message(Message::error(format!("{e}\n")));
            caps.message(Message::error(format!(
                "Task '{task_name}' failed to run\n"
            )));
        }
        println!("\n──────────Bake-Loop──────────\n");
    }
}

fn new_bake(caps: &Rc<CLICapabilities>) -> BakeViewModel {
    let bake_caps = Rc::clone(caps);
    match BakeViewModel::new(bake_caps) {
        Ok(x) => x,
        Err(e) => {
            println!("{}", e);
            exit(1); // cleaner way to panic
        }
    }
}
