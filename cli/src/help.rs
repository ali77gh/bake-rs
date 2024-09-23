use colored::Colorize;

pub fn show_help() {
    println!(" {}\n", " How to use Bake: ".on_blue());
    println!("    {} {}\n", "bake --help".blue(), "to see this message");
    println!(
        "    {} {}\n",
        "bake --version".blue(),
        "to see bake version"
    );
    println!(
        "    {} {}\n",
        "bake --update".blue(),
        "to open bake on github"
    );
    println!(
        "    {} {}\n",
        "bake --show".blue(),
        "to see list of tasks with indexes and help messages"
    );

    println!(
        "    {} {}\n",
        "bake <COMMAND_NAME>".blue(),
        "to run command with name (run bake --show to see list of names)"
    );
    println!(
        "    {} {}",
        "bake <COMMAND_INDEX>".blue(),
        "to run command with index (run bake --show to see list of indexes)"
    );
}
