mod arg_parser;
mod capabilities;
mod show_tasks;

use core::viewmodel::message::Message;
use core::viewmodel::BakeViewModel;
use std::rc::Rc;

use arg_parser::get_args;
use capabilities::CLICapabilities;
use core::viewmodel::capabilities::Capabilities;

fn main() {
    let bake = BakeViewModel::new(Rc::new(CLICapabilities)).expect("bakefile not found");

    match get_args() {
        arg_parser::ParsedArgs::ShowTasks => show_tasks::show_tasks(bake.tasks()),
        arg_parser::ParsedArgs::Invalid => println!("invalid args. \ntry run 'bake --help'"), // TODO show help
        arg_parser::ParsedArgs::Command(x) => match bake.run_task(&x) {
            Ok(()) => CLICapabilities.message(Message::new(
                core::viewmodel::message::MessageType::BakeState,
                format!("Task '{}' finished successfully\n", x),
            )),
            Err(e) => {
                CLICapabilities.message(Message::new(
                    core::viewmodel::message::MessageType::Error,
                    format!("{}\n", e),
                ));
                CLICapabilities.message(Message::new(
                    core::viewmodel::message::MessageType::BakeState,
                    format!("Task '{}' failed to run\n", x),
                ));
            }
        },
    }
}
