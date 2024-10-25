use std::collections::HashMap;

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
    pub fn set_envs(&mut self, envs: &HashMap<String, String>) {
        for (key, value) in envs {
            // check if its already exist
            match std::env::var(key) {
                Ok(value) => {
                    self.previous_envs.insert(key.to_string(), Some(value));
                }
                Err(_) => {
                    self.previous_envs.insert(key.to_string(), None);
                }
            }

            std::env::set_var(key, value);
        }
    }

    /// resets envs to previous_envs states
    pub fn role_back(self) {
        for (key, value) in self.previous_envs {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }
}
