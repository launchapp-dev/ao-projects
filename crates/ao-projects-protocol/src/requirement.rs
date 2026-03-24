use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::common::impl_from_str_via_serde;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RequirementStatus {
    #[default]
    Draft,
    Refined,
    Planned,
    InProgress,
    Done,
    Deprecated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RequirementPriority {
    Must,
    #[default]
    Should,
    Could,
    Wont,
}

impl RequirementPriority {
    pub fn to_task_priority(&self) -> crate::Priority {
        match self {
            Self::Must => crate::Priority::High,
            Self::Should => crate::Priority::Medium,
            Self::Could => crate::Priority::Low,
            Self::Wont => crate::Priority::Low,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RequirementType {
    Product,
    Functional,
    NonFunctional,
    Technical,
    #[default]
    Other,
}

impl_from_str_via_serde!(RequirementStatus);
impl_from_str_via_serde!(RequirementPriority);
impl_from_str_via_serde!(RequirementType);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequirementComment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default)]
    pub requirement_type: RequirementType,
    #[serde(default)]
    pub priority: RequirementPriority,
    #[serde(default)]
    pub status: RequirementStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub linked_task_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comments: Vec<RequirementComment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequirementFilter {
    #[serde(default)]
    pub status: Option<RequirementStatus>,
    #[serde(default)]
    pub priority: Option<RequirementPriority>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub requirement_type: Option<RequirementType>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub linked_task_id: Option<String>,
    #[serde(default)]
    pub search_text: Option<String>,
}
