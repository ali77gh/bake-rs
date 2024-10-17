use colored::Colorize;

pub fn show_help() {
    println!(" {}\n", " How to use Bake: ".on_blue());
    println!("    {} to see this message\n", "bake --help".blue());
    println!(
        "    {} to see bake version\n",
        "bake --version".blue()
    );
    println!(
        "    {} to open bake on github\n",
        "bake --update".blue()
    );
    println!(
        "    {} to see list of tasks with indexes and help messages\n",
        "bake --show".blue()
    );

    println!(
        "    {} to run command with name (run bake --show to see list of names)\n",
        "bake <COMMAND_NAME>".blue()
    );
    println!(
        "    {} to run command with index (run bake --show to see list of indexes)",
        "bake <COMMAND_INDEX>".blue()
    );
}
