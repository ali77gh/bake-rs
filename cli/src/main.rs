mod arg_parser;
mod capabilities;
mod help;
mod show_tasks;

use core::util::update::update;
use core::util::version::VERSION;
use core::viewmodel::message::Message;
use core::viewmodel::BakeViewModel;
use std::process::exit;
use std::rc::Rc;

use arg_parser::{get_args, ParsedArgs};
use capabilities::CLICapabilities;
use colored::Colorize;
use core::viewmodel::capabilities::Capabilities;
use help::show_help;

fn main() {
    println!(
        "{} {}\n",
        " ▶ Bake".bright_yellow(),
        "version: ".to_string() + VERSION
    );
    match get_args() {
        ParsedArgs::ShowTasks => show_tasks::show_tasks(bake(CLICapabilities::default()).tasks()),
        ParsedArgs::Invalid => println!("invalid args. \ntry run 'bake --help'"), // TODO show help
        ParsedArgs::Command(x, non_interactive) => {
            match bake(CLICapabilities { non_interactive }).run_task(&x) {
                Ok(()) => {}
                Err(e) => {
                    CLICapabilities::default().message(Message::error(format!("{e}\n")));
                    CLICapabilities::default()
                        .message(Message::error(format!("Task '{x}' failed to run\n")));
                }
            }
        }
        ParsedArgs::Version => {} // already printed
        ParsedArgs::Update => update(&CLICapabilities::default()),
        ParsedArgs::Help => show_help(),
    }
}

fn bake(cap: CLICapabilities) -> BakeViewModel {
    match BakeViewModel::new(Rc::new(cap)) {
        Ok(x) => x,
        Err(e) => {
            println!("{}", e);
            exit(1); // cleaner way to panic
        }
    }
}
