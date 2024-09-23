mod arg_parser;
mod capabilities;
mod help;
mod show_tasks;

use core::util::update::update;
use core::viewmodel::BakeViewModel;
use core::{util::version::show_version, viewmodel::message::Message};
use std::process::exit;
use std::rc::Rc;

use arg_parser::{get_args, ParsedArgs};
use capabilities::CLICapabilities;
use core::viewmodel::capabilities::Capabilities;
use help::show_help;

fn main() {
    match get_args() {
        ParsedArgs::ShowTasks => show_tasks::show_tasks(bake().tasks()),
        ParsedArgs::Invalid => println!("invalid args. \ntry run 'bake --help'"), // TODO show help
        ParsedArgs::Command(x) => match bake().run_task(&x) {
            Ok(()) => {}
            Err(e) => {
                CLICapabilities.message(Message::error(format!("{e}\n")));
                CLICapabilities.message(Message::error(format!("Task '{x}' failed to run\n")));
            }
        },
        ParsedArgs::Version => show_version(),
        ParsedArgs::Update => update(&CLICapabilities),
        ParsedArgs::Help => show_help(),
    }
}

fn bake() -> BakeViewModel {
    match BakeViewModel::new(Rc::new(CLICapabilities)) {
        Ok(x) => x,
        Err(e) => {
            println!("{}", e);
            exit(1); // cleaner way to panic
        }
    }
}
