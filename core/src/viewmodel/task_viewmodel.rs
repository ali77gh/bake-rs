use std::rc::Rc;

use crate::{
    model::{param::Param, task::Task},
    util::{measure_execution_time::measure_execution_time_result, ordered_map::OrderedMap},
};

use super::{
    capabilities::Capabilities, env_validator::validate_envs, message::Message, BakeViewModel,
};

/// contains [Task] yaml model but it actually contains logic (dependency checks and command running)
/// And use [Capabilities] for no side effects
pub struct TaskViewModel {
    capabilities: Rc<dyn Capabilities>,
    task: Task,
}

impl TaskViewModel {
    pub fn new(capabilities: Rc<dyn Capabilities>, task: Task) -> Self {
        Self { capabilities, task }
    }

    /// static function to generate hashmap for [BakeViewModel]
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

    /// install dependencies of task
    /// env checks
    /// run commands
    /// and show some messages including task time
    pub fn run(&self, bake_view_model: &BakeViewModel) -> Result<(), String> {
        // install dependencies of task
        // *this is recursive*
        bake_view_model.install_dependencies(self.dependencies())?;

        validate_envs(Rc::clone(&self.capabilities), self)?;

        self.capabilities.message(Message::bake_state(format!(
            "task '{}' is running...\n",
            self.name()
        )));

        let (_, duration) =
            measure_execution_time_result(|| bake_view_model.run_commands(&self.task.commands()?))?;

        self.capabilities.message(Message::bake_state(format!(
            "Task '{}' finished successfully. time: {}ms\n",
            self.name(),
            duration.as_millis()
        )));
        Ok(())
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
