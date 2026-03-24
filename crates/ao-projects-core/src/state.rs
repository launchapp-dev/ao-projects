use ao_projects_protocol::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProjectState {
    #[serde(default)]
    pub tasks: HashMap<String, Task>,
    #[serde(default)]
    pub requirements: HashMap<String, Requirement>,

    #[serde(skip)]
    pub dirty_tasks: HashSet<String>,
    #[serde(skip)]
    pub dirty_requirements: HashSet<String>,
    #[serde(skip)]
    pub all_tasks_dirty: bool,
    #[serde(skip)]
    pub all_requirements_dirty: bool,
}

impl ProjectState {
    pub fn next_task_id(&self) -> String {
        next_sequential_id(self.tasks.keys(), "TASK-")
    }

    pub fn next_requirement_id(&self) -> String {
        next_sequential_id(self.requirements.keys(), "REQ-")
    }
}

fn next_sequential_id<'a, I>(existing: I, prefix: &str) -> String
where
    I: Iterator<Item = &'a String>,
{
    let next = existing
        .filter_map(|id| id.strip_prefix(prefix))
        .filter_map(|seq| seq.parse::<u32>().ok())
        .max()
        .map_or(1, |max| max.saturating_add(1));
    format!("{}{:03}", prefix, next)
}
