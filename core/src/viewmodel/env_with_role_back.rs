use std::{collections::HashMap, rc::Rc};

use super::capabilities::Capabilities;

/// This struct can save env current state and role back after you run you'r shell commands
///
/// Example:
/// let mut env_with_role_back = EnvWithRoleBack::new();
/// env_with_role_back.set_envs(fc.params());
/// let r = self.run_task(fc.function());
/// env_with_role_back.role_back();
pub struct EnvWithRoleBack {
    previous_envs: HashMap<String, Option<String>>,
}

impl Default for EnvWithRoleBack {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvWithRoleBack {
    pub fn new() -> Self {
        EnvWithRoleBack {
            previous_envs: HashMap::new(),
        }
    }

    /// saves previous state of overwrote envs
    /// also sets envs as system envs
    pub fn set_envs(&mut self, caps: Rc<dyn Capabilities>, envs: &HashMap<String, String>) {
        for (key, value) in envs {
            // check if its already exist
            match caps.get_env(key) {
                Some(value) => {
                    self.previous_envs.insert(key.to_string(), Some(value));
                }
                None => {
                    self.previous_envs.insert(key.to_string(), None);
                }
            }

            caps.set_env(key, value);
        }
    }

    /// resets envs to previous_envs states
    pub fn role_back(self, caps: Rc<dyn Capabilities>) {
        for (key, value) in self.previous_envs {
            match value {
                Some(value) => caps.set_env(key.as_str(), value.as_str()),
                None => caps.remove_env(key.as_str()),
            }
        }
    }
}
