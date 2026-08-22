use std::{rc::Rc, time::Duration};

use crate::{
    model::{
        command::Command, end_handler::EndHandler, function_call::FunctionCall, param::Param,
        task::Task,
    },
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

    pub fn run(&self, bake_view_model: &BakeViewModel) -> Result<(), String> {
        if !self.task.keep_alive() {
            self.inner_run(bake_view_model)
        } else {
            loop {
                // user asked to stop (example: kill button in web app)
                if self.capabilities.should_abort() {
                    return Err(format!("task '{}' aborted", self.name()));
                }
                let result = self.inner_run(bake_view_model);
                // stop silently when the run was aborted (no restart message)
                if self.capabilities.should_abort() {
                    return Err(format!("task '{}' aborted", self.name()));
                }
                const RESTARTING_TEXT: &str = "restarting because of the keep_alive:true";
                match result {
                    Ok(_) => self.capabilities.message(Message::bake_state(format!(
                        "task '{}' done, {}\n",
                        self.name(),
                        RESTARTING_TEXT
                    ))),
                    Err(e) => {
                        self.capabilities.message(Message::error(format!("{e}, ")));
                        self.capabilities
                            .message(Message::bake_state(format!("{}\n", RESTARTING_TEXT)));
                    }
                }
            }
        }
    }

    /// install dependencies of task
    /// env checks
    /// run commands
    /// and show some messages including task time
    fn inner_run(&self, bake_view_model: &BakeViewModel) -> Result<(), String> {
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

        // here after doing role back we check the result
        match (r, self.task.on_success(), self.task.on_error()) {
            (Ok((_, d)), None, _) => {
                // success but no on_success
                self.print_time(d);
                if let Some(eh) = self.task.on_end() {
                    self.handle_end(bake_view_model, eh?, EndHandler::OnEnd);
                }
            }
            (Err(e), _, None) => {
                // error but no on_error
                if let Some(eh) = self.task.on_end() {
                    self.print_error(&e);
                    self.handle_end(bake_view_model, eh?, EndHandler::OnEnd);
                } else {
                    return Err(e);
                }
            }
            (Ok((_, d)), Some(eh), _) => {
                // success with on_success
                self.print_time(d);
                self.handle_end(bake_view_model, eh?, EndHandler::OnSuccess);
                if let Some(eh) = self.task.on_end() {
                    self.handle_end(bake_view_model, eh?, EndHandler::OnEnd);
                }
            }
            (Err(e), _, Some(eh)) => {
                // error with on_error
                self.print_error(&e);
                self.handle_end(bake_view_model, eh?, EndHandler::OnError);
                if let Some(eh) = self.task.on_end() {
                    self.handle_end(bake_view_model, eh?, EndHandler::OnEnd);
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

    fn handle_end(
        &self,
        bake_view_model: &BakeViewModel,
        function_call: FunctionCall,
        end_handler: EndHandler,
    ) {
        self.capabilities.message(Message::bake_state(format!(
            "task '{}' {}: calls function: '{}'\n",
            self.name(),
            end_handler,
            function_call
        )));
        let _ = bake_view_model.run_command(
            &Command::FunctionCall(function_call),
            self.task.working_directory(),
        );
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

    /// list of commands of this task
    pub fn commands(&self) -> Result<Vec<Command>, String> {
        self.task.commands()
    }

    pub fn working_directory(&self) -> Option<&str> {
        self.task.working_directory()
    }

    pub fn keep_alive(&self) -> bool {
        self.task.keep_alive()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::viewmodel::message::Message;

    struct AbortCap {
        executions: AtomicUsize,
    }

    impl Capabilities for AbortCap {
        fn read_file(&self, file_name: &str) -> Option<String> {
            if file_name != crate::viewmodel::BAKE_FILE_NAME {
                return None;
            }
            Some(
                "tasks:
  - name: loop
    keep_alive: true
    commands: [ echo hi ]
"
                .to_string(),
            )
        }

        fn execute(&self, _: &str, _: Option<&str>) -> bool {
            self.executions.fetch_add(1, Ordering::SeqCst);
            true
        }

        fn open_link(&self, _: &str) {}

        fn message(&self, _: Message) {}

        fn input(&self) -> Option<String> {
            None
        }

        fn set_env(&self, _: &str, _: &str) {}

        fn get_env(&self, _: &str) -> Option<String> {
            None
        }

        fn remove_env(&self, _: &str) {}

        /// abort as soon as the first run finished
        fn should_abort(&self) -> bool {
            self.executions.load(Ordering::SeqCst) > 0
        }
    }

    #[test]
    fn keep_alive_stops_on_abort() {
        let cap = Rc::new(AbortCap {
            executions: AtomicUsize::new(0),
        });
        let bake = BakeViewModel::new(Rc::clone(&cap) as Rc<dyn Capabilities>).unwrap();
        let r = bake.run_task("loop");
        assert!(r.is_err());
        assert_eq!(
            cap.executions.load(Ordering::SeqCst),
            1,
            "keep_alive should not restart after abort"
        );
    }
}
