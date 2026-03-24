use anyhow::Result;
use ao_projects_protocol::*;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::ProjectState;

pub struct TaskService {
    state: Arc<RwLock<ProjectState>>,
}

impl TaskService {
    pub fn new(state: Arc<RwLock<ProjectState>>) -> Self {
        Self { state }
    }

    pub async fn list(&self, filter: Option<TaskFilter>) -> Result<Vec<OrchestratorTask>> {
        let state = self.state.read().await;
        let mut tasks: Vec<OrchestratorTask> = state.tasks.values().cloned().collect();
        if let Some(f) = filter {
            tasks.retain(|t| task_matches_filter(t, &f));
        }
        sort_tasks_by_priority(&mut tasks);
        Ok(tasks)
    }

    pub async fn get(&self, id: &str) -> Result<OrchestratorTask> {
        let state = self.state.read().await;
        state.tasks.get(id).cloned().ok_or_else(|| anyhow::anyhow!("task not found: {}", id))
    }

    pub async fn create(&self, input: TaskCreateInput) -> Result<OrchestratorTask> {
        let mut state = self.state.write().await;
        let id = state.next_task_id();
        let now = Utc::now();
        let task = OrchestratorTask {
            id: id.clone(),
            title: input.title,
            description: input.description,
            task_type: input.task_type.unwrap_or_default(),
            status: TaskStatus::Backlog,
            priority: input.priority.unwrap_or(Priority::Medium),
            risk: RiskLevel::default(),
            scope: Scope::default(),
            complexity: Complexity::default(),
            impact_area: Vec::new(),
            assignee: Assignee::default(),
            estimated_effort: None,
            linked_requirements: input.linked_requirements,
            linked_architecture_entities: input.linked_architecture_entities,
            dependencies: Vec::new(),
            checklist: Vec::new(),
            tags: input.tags,
            workflow_metadata: WorkflowMetadata::default(),
            worktree_path: None,
            branch_name: None,
            metadata: TaskMetadata {
                created_at: now,
                updated_at: now,
                created_by: input.created_by.unwrap_or_default(),
                updated_by: String::new(),
                started_at: None,
                completed_at: None,
                version: 1,
            },
            deadline: None,
            paused: false,
            cancelled: false,
            resolution: None,
            resource_requirements: ResourceRequirements::default(),
            consecutive_dispatch_failures: None,
            last_dispatch_failure_at: None,
            dispatch_history: Vec::new(),
            blocked_reason: None,
            blocked_at: None,
            blocked_phase: None,
            blocked_by: None,
        };
        state.tasks.insert(id.clone(), task.clone());
        state.dirty_tasks.insert(id);
        Ok(task)
    }

    pub async fn update(&self, id: &str, input: TaskUpdateInput) -> Result<OrchestratorTask> {
        let mut state = self.state.write().await;
        let task = state.tasks.get_mut(id).ok_or_else(|| anyhow::anyhow!("task not found: {}", id))?;

        if let Some(title) = input.title { task.title = title; }
        if let Some(desc) = input.description { task.description = desc; }
        if let Some(priority) = input.priority { task.priority = priority; }
        if let Some(tags) = input.tags { task.tags = tags; }
        if let Some(deadline) = input.deadline { task.deadline = Some(deadline); }
        if let Some(entities) = input.linked_architecture_entities { task.linked_architecture_entities = entities; }
        if let Some(status) = input.status {
            apply_task_status(task, status);
        }
        task.metadata.updated_at = Utc::now();
        task.metadata.updated_by = input.updated_by.unwrap_or_default();
        task.metadata.version += 1;

        let task = task.clone();
        state.dirty_tasks.insert(id.to_string());
        Ok(task)
    }

    pub async fn replace(&self, task: OrchestratorTask) -> Result<OrchestratorTask> {
        let mut state = self.state.write().await;
        let id = task.id.clone();
        state.tasks.insert(id.clone(), task.clone());
        state.dirty_tasks.insert(id);
        Ok(task)
    }

    pub async fn set_status(&self, id: &str, status: TaskStatus) -> Result<OrchestratorTask> {
        let mut state = self.state.write().await;
        let task = state.tasks.get_mut(id).ok_or_else(|| anyhow::anyhow!("task not found: {}", id))?;
        apply_task_status(task, status);
        task.metadata.updated_at = Utc::now();
        task.metadata.version += 1;
        let task = task.clone();
        state.dirty_tasks.insert(id.to_string());
        Ok(task)
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        let mut state = self.state.write().await;
        state.tasks.remove(id).ok_or_else(|| anyhow::anyhow!("task not found: {}", id))?;
        state.all_tasks_dirty = true;
        Ok(())
    }

    pub async fn add_checklist_item(&self, id: &str, description: String) -> Result<OrchestratorTask> {
        let mut state = self.state.write().await;
        let task = state.tasks.get_mut(id).ok_or_else(|| anyhow::anyhow!("task not found: {}", id))?;
        let item = ChecklistItem {
            id: uuid::Uuid::new_v4().to_string(),
            description,
            completed: false,
            created_at: Utc::now(),
            completed_at: None,
        };
        task.checklist.push(item);
        task.metadata.updated_at = Utc::now();
        task.metadata.version += 1;
        let task = task.clone();
        state.dirty_tasks.insert(id.to_string());
        Ok(task)
    }

    pub async fn update_checklist_item(&self, id: &str, item_id: &str, completed: bool) -> Result<OrchestratorTask> {
        let mut state = self.state.write().await;
        let task = state.tasks.get_mut(id).ok_or_else(|| anyhow::anyhow!("task not found: {}", id))?;
        let item = task.checklist.iter_mut().find(|i| i.id == item_id)
            .ok_or_else(|| anyhow::anyhow!("checklist item not found: {}", item_id))?;
        item.completed = completed;
        item.completed_at = if completed { Some(Utc::now()) } else { None };
        task.metadata.updated_at = Utc::now();
        task.metadata.version += 1;
        let task = task.clone();
        state.dirty_tasks.insert(id.to_string());
        Ok(task)
    }

    pub async fn add_dependency(&self, id: &str, dep_id: &str, dep_type: DependencyType) -> Result<OrchestratorTask> {
        let mut state = self.state.write().await;
        let task = state.tasks.get_mut(id).ok_or_else(|| anyhow::anyhow!("task not found: {}", id))?;
        if task.dependencies.iter().any(|d| d.task_id == dep_id) {
            anyhow::bail!("dependency already exists: {} -> {}", id, dep_id);
        }
        task.dependencies.push(TaskDependency { task_id: dep_id.to_string(), dependency_type: dep_type });
        task.metadata.updated_at = Utc::now();
        task.metadata.version += 1;
        let task = task.clone();
        state.dirty_tasks.insert(id.to_string());
        Ok(task)
    }

    pub async fn remove_dependency(&self, id: &str, dep_id: &str) -> Result<OrchestratorTask> {
        let mut state = self.state.write().await;
        let task = state.tasks.get_mut(id).ok_or_else(|| anyhow::anyhow!("task not found: {}", id))?;
        let before = task.dependencies.len();
        task.dependencies.retain(|d| d.task_id != dep_id);
        if task.dependencies.len() == before {
            anyhow::bail!("dependency not found: {} -> {}", id, dep_id);
        }
        task.metadata.updated_at = Utc::now();
        task.metadata.version += 1;
        let task = task.clone();
        state.dirty_tasks.insert(id.to_string());
        Ok(task)
    }

    pub async fn statistics(&self) -> Result<TaskStatistics> {
        let state = self.state.read().await;
        let mut by_status = std::collections::HashMap::new();
        let mut by_priority = std::collections::HashMap::new();
        let mut by_type = std::collections::HashMap::new();
        let (mut in_progress, mut blocked, mut completed) = (0, 0, 0);

        for task in state.tasks.values() {
            *by_status.entry(task.status.to_string()).or_insert(0) += 1;
            *by_priority.entry(task.priority.to_string()).or_insert(0) += 1;
            *by_type.entry(task.task_type.to_string()).or_insert(0) += 1;
            match task.status {
                TaskStatus::InProgress => in_progress += 1,
                TaskStatus::Blocked => blocked += 1,
                TaskStatus::Done | TaskStatus::Cancelled => completed += 1,
                _ => {}
            }
        }

        Ok(TaskStatistics {
            total: state.tasks.len(),
            by_status,
            by_priority,
            by_type,
            in_progress,
            blocked,
            completed,
        })
    }
}

fn task_matches_filter(task: &OrchestratorTask, filter: &TaskFilter) -> bool {
    if let Some(ref s) = filter.status {
        if &task.status != s { return false; }
    }
    if let Some(ref p) = filter.priority {
        if &task.priority != p { return false; }
    }
    if let Some(ref t) = filter.task_type {
        if &task.task_type != t { return false; }
    }
    if let Some(ref req) = filter.linked_requirement {
        if !task.linked_requirements.contains(req) { return false; }
    }
    if let Some(ref tags) = filter.tags {
        if !tags.is_empty() && !tags.iter().any(|t| task.tags.contains(t)) { return false; }
    }
    if let Some(ref search) = filter.search_text {
        let s = search.to_lowercase();
        let matches = task.title.to_lowercase().contains(&s)
            || task.description.to_lowercase().contains(&s)
            || task.id.to_lowercase().contains(&s);
        if !matches { return false; }
    }
    true
}

fn sort_tasks_by_priority(tasks: &mut [OrchestratorTask]) {
    tasks.sort_by(|a, b| {
        a.priority.rank().cmp(&b.priority.rank())
            .then_with(|| b.metadata.updated_at.cmp(&a.metadata.updated_at))
            .then_with(|| a.id.cmp(&b.id))
    });
}

fn apply_task_status(task: &mut OrchestratorTask, status: TaskStatus) {
    let now = Utc::now();
    match &status {
        TaskStatus::InProgress => {
            task.metadata.started_at = Some(now);
            task.paused = false;
        }
        TaskStatus::Done => {
            task.metadata.completed_at = Some(now);
        }
        TaskStatus::Blocked => {
            task.blocked_at = Some(now);
            task.paused = true;
        }
        TaskStatus::Ready => {
            task.paused = false;
            task.blocked_reason = None;
            task.blocked_at = None;
            task.blocked_by = None;
            task.blocked_phase = None;
        }
        TaskStatus::Cancelled => {
            task.metadata.completed_at = Some(now);
            task.cancelled = true;
        }
        _ => {}
    }
    task.status = status;
}
