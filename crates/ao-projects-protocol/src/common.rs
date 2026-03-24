use serde::{Deserialize, Serialize};

macro_rules! impl_from_str_via_serde {
    ($ty:ty) => {
        impl std::str::FromStr for $ty {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let normalized = s.trim().to_lowercase().replace('-', "_");
                let quoted = format!("\"{}\"", normalized);
                serde_json::from_str(&quoted)
                    .map_err(|_| format!("invalid value: '{}' for {}", s, stringify!($ty)))
            }
        }
        impl std::fmt::Display for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let s = serde_json::to_string(self).unwrap_or_default();
                write!(f, "{}", s.trim_matches('"'))
            }
        }
    };
}

pub(crate) use impl_from_str_via_serde;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Critical,
    High,
    #[default]
    Medium,
    Low,
}

impl Priority {
    pub fn rank(&self) -> u8 {
        match self {
            Priority::Critical => 0,
            Priority::High => 1,
            Priority::Medium => 2,
            Priority::Low => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Complexity {
    High,
    #[default]
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Large,
    #[default]
    Medium,
    Small,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    High,
    #[default]
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Assignee {
    Agent {
        role: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
    },
    Human {
        user_id: String,
    },
    #[default]
    Unassigned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactArea {
    Frontend,
    Backend,
    Database,
    Api,
    Infrastructure,
    Docs,
    Tests,
    CiCd,
}

impl_from_str_via_serde!(Priority);
impl_from_str_via_serde!(Complexity);
impl_from_str_via_serde!(Scope);
impl_from_str_via_serde!(RiskLevel);
impl_from_str_via_serde!(ImpactArea);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}
