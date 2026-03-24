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

    pub async fn list(&self, filter: Option<TaskFilter>) -> Result<Vec<Task>> {
        let state = self.state.read().await;
        let mut tasks: Vec<Task> = state.tasks.values().cloned().collect();
        if let Some(f) = filter {
            tasks.retain(|t| task_matches_filter(t, &f));
        }
        sort_tasks_by_priority(&mut tasks);
        Ok(tasks)
    }

    pub async fn get(&self, id: &str) -> Result<Task> {
        let state = self.state.read().await;
        state.tasks.get(id).cloned().ok_or_else(|| anyhow::anyhow!("task not found: {}", id))
    }

    pub async fn create(&self, input: TaskCreateInput) -> Result<Task> {
        let mut state = self.state.write().await;
        let id = state.next_task_id();
        let now = Utc::now();
        let task = Task {
            id: id.clone(),
            title: input.title,
            description: input.description,
            task_type: input.task_type.unwrap_or_default(),
            status: TaskStatus::Backlog,
            priority: input.priority.unwrap_or_default(),
            risk: RiskLevel::default(),
            scope: Scope::default(),
            complexity: Complexity::default(),
            impact_area: None,
            assignee: Assignee::default(),
            tags: input.tags,
            checklist: Vec::new(),
            dependencies: Vec::new(),
            linked_requirements: input.linked_requirements,
            metadata: TaskMetadata {
                created_at: Some(now),
                updated_at: Some(now),
                version: 1,
                ..Default::default()
            },
            blocked_reason: None,
            blocked_at: None,
            blocked_by: None,
            paused: false,
            dispatch_history: Vec::new(),
            consecutive_dispatch_failures: 0,
            deadline: None,
        };
        state.tasks.insert(id.clone(), task.clone());
        state.dirty_tasks.insert(id);
        Ok(task)
    }

    pub async fn update(&self, id: &str, input: TaskUpdateInput) -> Result<Task> {
        let mut state = self.state.write().await;
        let task = state.tasks.get_mut(id).ok_or_else(|| anyhow::anyhow!("task not found: {}", id))?;

        if let Some(title) = input.title { task.title = title; }
        if let Some(desc) = input.description { task.description = Some(desc); }
        if let Some(priority) = input.priority { task.priority = priority; }
        if let Some(assignee) = input.assignee { task.assignee = assignee; }
        if let Some(tags) = input.tags { task.tags = tags; }
        if let Some(deadline) = input.deadline { task.deadline = Some(deadline); }
        if let Some(status) = input.status {
            apply_task_status(task, status)?;
        }
        task.metadata.updated_at = Some(Utc::now());
        task.metadata.version += 1;

        let task = task.clone();
        state.dirty_tasks.insert(id.to_string());
        Ok(task)
    }

    pub async fn set_status(&self, id: &str, status: TaskStatus) -> Result<Task> {
        let mut state = self.state.write().await;
        let task = state.tasks.get_mut(id).ok_or_else(|| anyhow::anyhow!("task not found: {}", id))?;
        apply_task_status(task, status)?;
        task.metadata.updated_at = Some(Utc::now());
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

    pub async fn add_checklist_item(&self, id: &str, description: String) -> Result<Task> {
        let mut state = self.state.write().await;
        let task = state.tasks.get_mut(id).ok_or_else(|| anyhow::anyhow!("task not found: {}", id))?;
        let item = ChecklistItem {
            id: uuid::Uuid::new_v4().to_string(),
            description,
            completed: false,
            created_at: Some(Utc::now()),
            completed_at: None,
        };
        task.checklist.push(item);
        task.metadata.updated_at = Some(Utc::now());
        task.metadata.version += 1;
        let task = task.clone();
        state.dirty_tasks.insert(id.to_string());
        Ok(task)
    }

    pub async fn update_checklist_item(&self, id: &str, item_id: &str, completed: bool) -> Result<Task> {
        let mut state = self.state.write().await;
        let task = state.tasks.get_mut(id).ok_or_else(|| anyhow::anyhow!("task not found: {}", id))?;
        let item = task.checklist.iter_mut().find(|i| i.id == item_id)
            .ok_or_else(|| anyhow::anyhow!("checklist item not found: {}", item_id))?;
        item.completed = completed;
        item.completed_at = if completed { Some(Utc::now()) } else { None };
        task.metadata.updated_at = Some(Utc::now());
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
            *by_status.entry(format!("{:?}", task.status).to_lowercase()).or_insert(0) += 1;
            *by_priority.entry(format!("{:?}", task.priority).to_lowercase()).or_insert(0) += 1;
            *by_type.entry(format!("{:?}", task.task_type).to_lowercase()).or_insert(0) += 1;
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

fn task_matches_filter(task: &Task, filter: &TaskFilter) -> bool {
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
    if !filter.tags.is_empty() {
        if !filter.tags.iter().any(|t| task.tags.contains(t)) { return false; }
    }
    if let Some(ref search) = filter.search_text {
        let s = search.to_lowercase();
        let matches = task.title.to_lowercase().contains(&s)
            || task.description.as_deref().unwrap_or("").to_lowercase().contains(&s)
            || task.id.to_lowercase().contains(&s);
        if !matches { return false; }
    }
    true
}

fn sort_tasks_by_priority(tasks: &mut [Task]) {
    tasks.sort_by(|a, b| {
        a.priority.rank().cmp(&b.priority.rank())
            .then_with(|| b.metadata.updated_at.cmp(&a.metadata.updated_at))
            .then_with(|| a.id.cmp(&b.id))
    });
}

fn apply_task_status(task: &mut Task, status: TaskStatus) -> Result<()> {
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
        }
        TaskStatus::Cancelled => {
            task.metadata.completed_at = Some(now);
        }
        _ => {}
    }
    task.status = status;
    Ok(())
}
