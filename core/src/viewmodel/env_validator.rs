use std::rc::Rc;

use crate::model::param::Param;

use super::{
    capabilities::Capabilities, env_with_role_back::EnvWithRoleBack, task_viewmodel::TaskViewModel,
};

/// It's just [validate_env] in a loop which stops iteration on error
pub fn validate_envs(
    cap: Rc<dyn Capabilities>,
    task: &TaskViewModel,
) -> Result<EnvWithRoleBack, String> {
    let mut env_with_role_back = EnvWithRoleBack::new();
    for env in task.params() {
        if let Err(e) = validate_env(cap.clone(), env, &mut env_with_role_back) {
            // we should role back what we get from user if something goes wrong
            env_with_role_back.role_back(cap);
            return Err(e);
        };
    }
    Ok(env_with_role_back)
}

/// Value in yaml consider as default value
/// So this function first tries to get value from env vars
/// and if value is not there then it tries to get default from yaml
/// and set it to process env vars
/// if not it will ask user for value
pub fn validate_env(
    cap: Rc<dyn Capabilities>,
    env: &Param,
    env_with_role_back: &mut EnvWithRoleBack,
) -> Result<(), String> {
    let key = env.name();
    let value = match cap.get_env(key) {
        Some(value) => value,
        None => match env.default() {
            Some(value) => {
                env_with_role_back.set_env(cap, key, value);
                value.to_string()
            }
            None => {
                let user_input = cap
                    .as_ref()
                    .ask_user(&format!(
                        "environment variable '{0}' not found enter {0}",
                        key
                    ))
                    .ok_or(format!("can't get environment variable '{0}'", key))?
                    .trim()
                    .to_string();
                env_with_role_back.set_env(cap, key, &user_input);
                user_input.to_string()
            }
        },
    };
    match env.validator() {
        Some(validator) => match validator?.validate(&value) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!(
                "environment variable validation error: '{}'='{}' ({})",
                key, value, e
            )),
        },
        None => Ok(()), // no validator
    }
}
