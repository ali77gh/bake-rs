use crate::model::param::Param;

use super::task_viewmodel::TaskViewModel;

pub fn validate_envs(task: &TaskViewModel) -> Result<(), String> {
    for env in task.params() {
        validate_env(env)?;
    }
    Ok(())
}

/// Value in yaml consider as default value
/// So this function first tries to get value from env vars
/// and if value is not there then it tries to get default from yaml
/// and set it to process env vars
pub fn validate_env(env: &Param) -> Result<(), String> {
    let key = env.name();
    let value = match std::env::var(key) {
        Ok(value) => value,
        Err(e) => match env.default() {
            Some(value) => {
                std::env::set_var(key, value); // load yaml value to env
                value.to_string()
            }
            None => {
                return Err(format!(
                    "environment variable error: '{}' not set ({})",
                    key, e
                ))
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
