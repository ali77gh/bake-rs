use std::rc::Rc;

use crate::model::task::Task;

use super::{capabilities::Capabilities, dependency_viewmodel::filter_commands};

pub struct TaskViewModel {
    capabilities: Rc<dyn Capabilities>,
    task: Task,
}

// TODO check dependencies are installed
impl TaskViewModel {
    pub fn new(capabilities: Rc<dyn Capabilities>, task: Task) -> Self {
        Self { capabilities, task }
    }

    pub fn run(&self) -> Result<(), String> {
        let commands = self.task.commands()?;

        // TODO remove this filter (you should run function calls too!)
        let commands = filter_commands(&commands);

        self.capabilities.execute_and_print_all(&commands)
    }

    pub fn name(&self) -> &str {
        self.task.name()
    }

    pub fn is(&self, name: &str) -> bool {
        self.task.name() == name
    }

    pub fn help_msg(&self) -> &Option<String> {
        self.task.help_msg()
    }

    pub fn dependencies(&self) -> &Vec<String> {
        const EMPTY: &Vec<String> = &vec![];
        match self.task.dependencies() {
            Some(x) => x,
            None => EMPTY,
        }
    }
}
