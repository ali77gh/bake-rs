use std::{rc::Rc, time::Duration};

use crate::{
    model::{command::Command, end_handler::EndHandler, param::Param, task::Task},
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
    pub fn run(
        &self,
        bake_view_model: &BakeViewModel,
        skip_end_handler: bool,
    ) -> Result<(), String> {
        // install dependencies of task
        // *this is recursive*
        bake_view_model.install_dependencies(self.dependencies())?;

        let env_role_back = validate_envs(Rc::clone(&self.capabilities), self)?;

        self.capabilities.message(Message::bake_state(format!(
            "task '{}' is running...\n",
            self.name()
        )));

        // *this is recursive*
        // we should do role back anyway (does not matter if task fails or not)
        // so we can not use '?' operator here
        // we should do this after role back
        let r = measure_execution_time_result(|| {
            bake_view_model.run_commands(&self.task.commands()?, self.task.working_directory())
        });

        env_role_back.role_back(self.capabilities.clone());

        if skip_end_handler {
            let (_, duration) = r?;
            self.print_time(duration);
        } else {
            // here after doing role back we check the result
            match (r, self.task.on_success(), self.task.on_error()) {
                (Ok((_, d)), None, _) => {
                    self.print_time(d);
                }
                (Err(e), _, None) => return Err(e),
                (Ok((_, d)), Some(eh), _) => {
                    self.print_time(d);
                    self.handle_end(bake_view_model, eh?);
                }
                (Err(e), _, Some(eh)) => {
                    self.print_error(&e);
                    self.handle_end(bake_view_model, eh?);
                }
            }
        }

        Ok(())
    }

    fn print_time(&self, duration: Duration) {
        self.capabilities.message(Message::bake_state(format!(
            "Task '{}' finished successfully. time: {}ms\n",
            self.name(),
            duration.as_millis()
        )));
    }

    fn print_error(&self, msg: &str) {
        self.capabilities.message(Message::error(format!(
            "Task '{}' failed. error message: {}\n",
            self.name(),
            msg
        )));
    }

    fn handle_end(&self, bake_view_model: &BakeViewModel, end_handler: EndHandler) {
        match end_handler {
            EndHandler::Restart => {
                // I don't like to do recursion here because it will stack overflow at some point and fail
                loop {
                    self.capabilities.message(Message::bake_state(format!(
                        "restarting task '{}'\n",
                        self.name()
                    )));
                    let _ = self.run(bake_view_model, true);
                }
            }
            EndHandler::Retry(retries) => {
                for i in 0..retries {
                    self.capabilities.message(Message::bake_state(format!(
                        "retrying for {}th time (retry limit is '{}')",
                        i, retries
                    ))); // TODO write a better nth function
                    let _ = self.run(bake_view_model, true);
                }
            }
            EndHandler::FunctionCall(function_call) => {
                self.capabilities.message(Message::bake_state(format!(
                    "on_error: '{}'\n",
                    function_call
                )));
                let _ = bake_view_model.run_command(
                    &Command::FunctionCall(function_call),
                    self.task.working_directory(),
                );
            }
        }
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
