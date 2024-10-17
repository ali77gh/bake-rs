use std::collections::HashMap;

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

    pub fn role_back(self) {
        for (key, value) in self.previous_envs {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }
}
