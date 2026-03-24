use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::common::*;

// --- TaskStatus: kebab-case with aliases, matches AO exactly ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    #[serde(alias = "todo")]
    Backlog,
    Ready,
    #[serde(alias = "in_progress", alias = "inprogress")]
    InProgress,
    Blocked,
    #[serde(alias = "on_hold", alias = "onhold")]
    OnHold,
    #[serde(alias = "completed")]
    Done,
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Backlog
    }
}

impl TaskStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::InProgress)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done | Self::Cancelled)
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Blocked | Self::OnHold)
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Backlog => "backlog",
            Self::Ready => "ready",
            Self::InProgress => "in-progress",
            Self::Blocked => "blocked",
            Self::OnHold => "on-hold",
            Self::Done => "done",
            Self::Cancelled => "cancelled",
        })
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_lowercase();
        Ok(match normalized.as_str() {
            "todo" | "backlog" => Self::Backlog,
            "ready" => Self::Ready,
            "in_progress" | "in-progress" => Self::InProgress,
            "done" | "completed" => Self::Done,
            "blocked" => Self::Blocked,
            "on_hold" | "on-hold" => Self::OnHold,
            "cancelled" => Self::Cancelled,
            _ => return Err(format!("unknown task status: {s}")),
        })
    }
}

// --- TaskType: lowercase with aliases ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    Feature,
    #[serde(alias = "bug")]
    Bugfix,
    #[serde(alias = "hot-fix")]
    Hotfix,
    Refactor,
    #[serde(alias = "documentation", alias = "doc")]
    Docs,
    #[serde(alias = "tests", alias = "testing")]
    Test,
    Chore,
    Experiment,
}

impl Default for TaskType {
    fn default() -> Self {
        Self::Feature
    }
}

impl TaskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Feature => "feature",
            Self::Bugfix => "bugfix",
            Self::Hotfix => "hotfix",
            Self::Refactor => "refactor",
            Self::Docs => "docs",
            Self::Test => "test",
            Self::Chore => "chore",
            Self::Experiment => "experiment",
        }
    }
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for TaskType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "feature" => Ok(Self::Feature),
            "bugfix" | "bug" => Ok(Self::Bugfix),
            "hotfix" | "hot-fix" => Ok(Self::Hotfix),
            "refactor" => Ok(Self::Refactor),
            "docs" | "doc" | "documentation" => Ok(Self::Docs),
            "test" | "tests" | "testing" => Ok(Self::Test),
            "chore" => Ok(Self::Chore),
            "experiment" => Ok(Self::Experiment),
            _ => Err(format!("unknown task type: {s}")),
        }
    }
}

// --- DependencyType: kebab-case ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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

// --- WorkflowMetadata ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub workflow_id: Option<String>,
    pub requires_design: bool,
    pub requires_architecture: bool,
    pub requires_qa: bool,
    pub requires_staging_deploy: bool,
    pub requires_production_deploy: bool,
}

impl Default for WorkflowMetadata {
    fn default() -> Self {
        Self {
            workflow_id: None,
            requires_design: false,
            requires_architecture: false,
            requires_qa: true,
            requires_staging_deploy: false,
            requires_production_deploy: false,
        }
    }
}

// --- ResourceRequirements ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub max_cpu_percent: Option<f32>,
    pub max_memory_mb: Option<u64>,
    pub requires_network: bool,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self { max_cpu_percent: None, max_memory_mb: None, requires_network: true }
    }
}

// --- TaskMetadata: non-optional timestamps, u32 version ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_by: String,
    #[serde(default)]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(default = "default_task_version")]
    pub version: u32,
}

const fn default_task_version() -> u32 {
    1
}

impl Default for TaskMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            created_by: String::new(),
            updated_by: String::new(),
            started_at: None,
            completed_at: None,
            version: 1,
        }
    }
}

// --- DispatchHistoryEntry: String timestamps to match AO ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchHistoryEntry {
    pub workflow_id: String,
    pub started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<f64>,
    pub outcome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}

// --- OrchestratorTask: exact match with AO ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorTask {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub task_type: TaskType,
    pub status: TaskStatus,
    #[serde(default)]
    pub blocked_reason: Option<String>,
    #[serde(default)]
    pub blocked_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub blocked_phase: Option<String>,
    #[serde(default)]
    pub blocked_by: Option<String>,
    pub priority: Priority,
    #[serde(default)]
    pub risk: RiskLevel,
    #[serde(default)]
    pub scope: Scope,
    #[serde(default)]
    pub complexity: Complexity,
    #[serde(default)]
    pub impact_area: Vec<ImpactArea>,
    #[serde(default)]
    pub assignee: Assignee,
    #[serde(default)]
    pub estimated_effort: Option<String>,
    #[serde(default)]
    pub linked_requirements: Vec<String>,
    #[serde(default)]
    pub linked_architecture_entities: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<TaskDependency>,
    #[serde(default)]
    pub checklist: Vec<ChecklistItem>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub workflow_metadata: WorkflowMetadata,
    #[serde(default)]
    pub worktree_path: Option<String>,
    #[serde(default)]
    pub branch_name: Option<String>,
    pub metadata: TaskMetadata,
    #[serde(default)]
    pub deadline: Option<String>,
    #[serde(default)]
    pub paused: bool,
    #[serde(default)]
    pub cancelled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(default)]
    pub resource_requirements: ResourceRequirements,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consecutive_dispatch_failures: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_dispatch_failure_at: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dispatch_history: Vec<DispatchHistoryEntry>,
}

// --- Input/Filter types matching AO ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskCreateInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub task_type: Option<TaskType>,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub linked_requirements: Vec<String>,
    #[serde(default)]
    pub linked_architecture_entities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub assignee: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub updated_by: Option<String>,
    #[serde(default)]
    pub deadline: Option<String>,
    #[serde(default)]
    pub linked_architecture_entities: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskFilter {
    pub task_type: Option<TaskType>,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub risk: Option<RiskLevel>,
    pub assignee_type: Option<String>,
    pub tags: Option<Vec<String>>,
    pub linked_requirement: Option<String>,
    pub linked_architecture_entity: Option<String>,
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
