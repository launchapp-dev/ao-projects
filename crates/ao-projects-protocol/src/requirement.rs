use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::common::RequirementPriority;

// --- RequirementType: kebab-case with aliases ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RequirementType {
    Product,
    Functional,
    #[serde(alias = "nonfunctional")]
    NonFunctional,
    Technical,
    Other,
}

impl Default for RequirementType {
    fn default() -> Self {
        Self::Other
    }
}

// --- RequirementStatus: kebab-case, all 11 variants ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RequirementStatus {
    #[default]
    Draft,
    Refined,
    Planned,
    #[serde(alias = "in_progress")]
    InProgress,
    Done,
    PoReview,
    EmReview,
    NeedsRework,
    Approved,
    Implemented,
    Deprecated,
}

impl std::fmt::Display for RequirementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Draft => "draft",
            Self::Refined => "refined",
            Self::Planned => "planned",
            Self::InProgress => "in-progress",
            Self::Done => "done",
            Self::PoReview => "po-review",
            Self::EmReview => "em-review",
            Self::NeedsRework => "needs-rework",
            Self::Approved => "approved",
            Self::Implemented => "implemented",
            Self::Deprecated => "deprecated",
        })
    }
}

impl std::str::FromStr for RequirementStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_lowercase().replace('_', "-");
        Ok(match normalized.as_str() {
            "draft" => Self::Draft,
            "refined" => Self::Refined,
            "planned" => Self::Planned,
            "in-progress" => Self::InProgress,
            "done" => Self::Done,
            "po-review" => Self::PoReview,
            "em-review" => Self::EmReview,
            "needs-rework" => Self::NeedsRework,
            "approved" => Self::Approved,
            "implemented" => Self::Implemented,
            "deprecated" => Self::Deprecated,
            _ => return Err(format!("unknown requirement status: {s}")),
        })
    }
}

// --- RequirementLinks: matches AO exactly ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequirementLinks {
    #[serde(default)]
    pub tasks: Vec<String>,
    #[serde(default)]
    pub workflows: Vec<String>,
    #[serde(default)]
    pub tests: Vec<String>,
    #[serde(default)]
    pub mockups: Vec<String>,
    #[serde(default)]
    pub flows: Vec<String>,
    #[serde(default)]
    pub related_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementComment {
    pub author: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub phase: Option<String>,
}

// --- RequirementItem: exact match with AO ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementItem {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub legacy_id: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(rename = "type", default)]
    pub requirement_type: Option<RequirementType>,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub priority: RequirementPriority,
    #[serde(default)]
    pub status: RequirementStatus,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub links: RequirementLinks,
    #[serde(default)]
    pub comments: Vec<RequirementComment>,
    #[serde(default)]
    pub relative_path: Option<String>,
    #[serde(default)]
    pub linked_task_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// --- Filter matching AO ---

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequirementFilter {
    #[serde(default)]
    pub status: Option<RequirementStatus>,
    #[serde(default)]
    pub priority: Option<RequirementPriority>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(rename = "type", default)]
    pub requirement_type: Option<RequirementType>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub linked_task_id: Option<String>,
    #[serde(default)]
    pub search_text: Option<String>,
}

// --- Input types ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequirementCreateInput {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<RequirementPriority>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub requirement_type: Option<RequirementType>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequirementUpdateInput {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<RequirementPriority>,
    #[serde(default)]
    pub status: Option<RequirementStatus>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub requirement_type: Option<RequirementType>,
    #[serde(default)]
    pub acceptance_criteria: Option<Vec<String>>,
    #[serde(default)]
    pub replace_acceptance_criteria: bool,
    #[serde(default)]
    pub linked_task_id: Option<String>,
}
