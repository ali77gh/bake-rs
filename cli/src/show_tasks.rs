use core::viewmodel::task_viewmodel::TaskViewModel;

pub fn show_tasks(tasks: &[TaskViewModel]) {
    if tasks.is_empty() {
        println!("there is no task in bakefile");
        return;
    }

    println!("Tasks:\n");
    for (i, task) in tasks.iter().enumerate() {
        println!(
            "* [{}] {} {}\n",
            i + 1,
            task.name(),
            match task.help_msg().clone() {
                Some(x) => format!("({})", x),
                None => "".to_string(),
            }
        );
    }
}
