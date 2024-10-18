use std::rc::Rc;

use crate::model::param::Param;

use super::{capabilities::Capabilities, task_viewmodel::TaskViewModel};

pub fn validate_envs(cap: Rc<dyn Capabilities>, task: &TaskViewModel) -> Result<(), String> {
    for env in task.params() {
        validate_env(cap.clone(), env)?;
    }
    Ok(())
}

/// Value in yaml consider as default value
/// So this function first tries to get value from env vars
/// and if value is not there then it tries to get default from yaml
/// and set it to process env vars
pub fn validate_env(cap: Rc<dyn Capabilities>, env: &Param) -> Result<(), String> {
    let key = env.name();
    let value = match std::env::var(key) {
        Ok(value) => value,
        Err(_) => match env.default() {
            Some(value) => {
                std::env::set_var(key, value); // load yaml value to env
                value.to_string()
            }
            None => {
                let user_input = cap
                    .as_ref()
                    .ask_user(&format!(
                        "environment variable '{0}' not found enter {0}",
                        key
                    ))
                    .trim()
                    .to_string();
                std::env::set_var(key, &user_input);
                user_input.to_string()
            }
        },
    };
    match env.validator() {
        Some(validator) => match validator.validate(&value) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!(
                "environment variable validation error: '{}'='{}' ({})",
                key, value, e
            )),
        },
        None => Ok(()), // no validator
    }
}
