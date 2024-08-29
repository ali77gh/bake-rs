use super::{dependency::Dependency, task::Task};

struct BakeFile {
    dependencies: Vec<Dependency>,
    tasks: Vec<Task>,
}
