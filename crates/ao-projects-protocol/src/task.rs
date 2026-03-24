use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::common::*;
use crate::common::impl_from_str_via_serde;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[default]
    Backlog,
    Ready,
    InProgress,
    Blocked,
    OnHold,
    Done,
    Cancelled,
}

impl TaskStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Ready | Self::InProgress | Self::Blocked | Self::OnHold)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done | Self::Cancelled)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    #[default]
    Feature,
    Bugfix,
    Hotfix,
    Refactor,
    Docs,
    Test,
    Chore,
    Experiment,
}

impl_from_str_via_serde!(TaskStatus);
impl_from_str_via_serde!(TaskType);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    BlocksBy,
    BlockedBy,
    RelatedTo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependency {
    pub task_id: String,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskMetadata {
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchHistoryEntry {
    pub workflow_id: String,
    pub workflow_ref: Option<String>,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_phase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub task_type: TaskType,
    #[serde(default)]
    pub status: TaskStatus,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub risk: RiskLevel,
    #[serde(default)]
    pub scope: Scope,
    #[serde(default)]
    pub complexity: Complexity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact_area: Option<ImpactArea>,
    #[serde(default)]
    pub assignee: Assignee,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checklist: Vec<ChecklistItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<TaskDependency>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub linked_requirements: Vec<String>,
    #[serde(default)]
    pub metadata: TaskMetadata,

    // Blocking state
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_by: Option<Vec<String>>,
    #[serde(default)]
    pub paused: bool,

    // Dispatch tracking
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dispatch_history: Vec<DispatchHistoryEntry>,
    #[serde(default)]
    pub consecutive_dispatch_failures: u32,

    // Scheduling
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskCreateInput {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub task_type: Option<TaskType>,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub linked_requirements: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskUpdateInput {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub status: Option<TaskStatus>,
    #[serde(default)]
    pub assignee: Option<Assignee>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub deadline: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskFilter {
    #[serde(default)]
    pub task_type: Option<TaskType>,
    #[serde(default)]
    pub status: Option<TaskStatus>,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub linked_requirement: Option<String>,
    #[serde(default)]
    pub search_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatistics {
    pub total: usize,
    pub by_status: std::collections::HashMap<String, usize>,
    pub by_priority: std::collections::HashMap<String, usize>,
    pub by_type: std::collections::HashMap<String, usize>,
    pub in_progress: usize,
    pub blocked: usize,
    pub completed: usize,
}
