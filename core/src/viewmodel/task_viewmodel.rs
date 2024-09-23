use std::rc::Rc;

use crate::{
    model::{param::Param, task::Task},
    util::ordered_map::OrderedMap,
};

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
    ) -> OrderedMap<String, TaskViewModel> {
        let mut map = OrderedMap::new();
        for task in tasks {
            let task_vm = TaskViewModel::new(Rc::clone(&capabilities), task.clone());
            map.insert(task.name().to_string(), task_vm);
        }
        map
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
        self.task.dependencies()
    }

    pub fn params(&self) -> &[Param] {
        self.task.envs()
    }
}
