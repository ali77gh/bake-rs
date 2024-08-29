use super::{dependency::Dependency, param::Param, task::Task};

struct BakeFile {
    global_env_vars: Vec<Param>,
    dependencies: Vec<Dependency>,
    tasks: Vec<Task>,
}
