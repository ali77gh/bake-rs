use colored::Colorize;

pub fn show_help() {
    println!(" {}\n", " How to use Bake: ".on_bright_yellow().black());
    println!(
        "    {} to see this message\n",
        "bake --help".bright_yellow()
    );
    println!(
        "    {} to see bake version\n",
        "bake --version".bright_yellow()
    );
    println!(
        "    {} to open bake on github\n",
        "bake --update".bright_yellow()
    );
    println!(
        "    {} to see list of tasks with indexes and help messages\n",
        "bake --show".bright_yellow()
    );

    println!(
        "    {} to run command with name (run bake --show to see list of names)\n",
        "bake <COMMAND_NAME>".bright_yellow()
    );
    println!(
        "    {} to run command with index (run bake --show to see list of indexes)",
        "bake <COMMAND_INDEX>".bright_yellow()
    );
}
