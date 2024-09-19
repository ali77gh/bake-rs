use std::{collections::HashMap, rc::Rc};

use crate::model::task::Task;

use super::{capabilities::Capabilities, BakeViewModel};

pub struct TaskViewModel {
    capabilities: Rc<dyn Capabilities>,
    task: Task,
}

impl TaskViewModel {
    pub fn new(capabilities: Rc<dyn Capabilities>, task: Task) -> Self {
        Self { capabilities, task }
    }

    pub fn hashmap_from_tasks(
        capabilities: Rc<dyn Capabilities>,
        tasks: &[Task],
    ) -> HashMap<String, TaskViewModel> {
        tasks
            .iter()
            .map(|task| {
                (
                    task.name().to_string(),
                    TaskViewModel::new(Rc::clone(&capabilities), task.clone()),
                )
            })
            .collect::<HashMap<String, TaskViewModel>>()
    }

    pub fn run(&self, bake_view_model: &BakeViewModel) -> Result<(), String> {
        let commands = self.task.commands()?;
        bake_view_model.run_commands(&commands)
    }

    pub fn name(&self) -> &str {
        self.task.name()
    }

    pub fn is(&self, name: &str) -> bool {
        self.task.name() == name
    }

    pub fn help_msg(&self) -> Option<&str> {
        self.task.help_msg()
    }

    pub fn dependencies(&self) -> &[String] {
        const EMPTY: &Vec<String> = &vec![];
        match self.task.dependencies() {
            Some(x) => x,
            None => EMPTY,
        }
    }
}
