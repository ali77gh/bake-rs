use core::viewmodel::task_viewmodel::TaskViewModel;

use colored::Colorize;

pub fn show_tasks(tasks: &[TaskViewModel]) {
    if tasks.is_empty() {
        println!("there is no task in bakefile");
        return;
    }

    println!("{}\n", " Tasks: ".on_bright_yellow().black());
    for (i, task) in tasks.iter().enumerate() {
        println!(
            "  {} {} {} {}\n",
            format!(" {} ", (i + 1)).on_bright_yellow().black(),
            "▶".bright_yellow(),
            task.name().bold().underline(),
            match task.help_msg() {
                Some(x) => format!("({})", x.italic()),
                None => "".to_string(),
            }
        );
    }
}
