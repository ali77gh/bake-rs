use core::viewmodel::{
    capabilities::Capabilities, message::Message, BakeViewModel, BAKE_FILE_NAME,
};
use std::{process::exit, rc::Rc, time::SystemTime};

use crate::{capabilities::CLICapabilities, show_tasks::show_tasks};

pub fn start_repl() {
    let caps = Rc::new(CLICapabilities::default());

    loop {
        let mut bake = new_bake(&caps);
        let last_mod = get_bakefile_last_modification();
        show_tasks(bake.tasks());
        let task_name = match caps.ask_user("which task do you want to run (enter name or index)") {
            Some(x) => x,
            None => {
                println!("Bake: bye bye!");
                exit(1);
            }
        };

        if last_mod != get_bakefile_last_modification() {
            caps.message(Message::bake_state(
                "bakefile change detected! reloading bakefile...\n",
            ));
            // read the bakefile again to get new changes happened during ask_user
            bake = new_bake(&caps);
        }

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

fn get_bakefile_last_modification() -> SystemTime {
    std::fs::metadata(BAKE_FILE_NAME)
        .unwrap()
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH)
}
