use super::{dependency::Dependency, task::Task};

struct bakeFile {
    dependencies: Vec<Dependency>,
    tasks: Vec<Task>,
}
