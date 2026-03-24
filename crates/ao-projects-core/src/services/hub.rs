use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use ao_projects_store::{read_json_or_default, write_json_atomic, scoped_state_root};
use crate::state::ProjectState;
use super::{TaskService, RequirementService};

pub struct ProjectHub {
    state: Arc<RwLock<ProjectState>>,
    state_path: PathBuf,
    task_service: TaskService,
    requirement_service: RequirementService,
}

impl ProjectHub {
    pub fn load(project_root: &Path) -> Result<Self> {
        let state_dir = scoped_state_root(project_root);
        std::fs::create_dir_all(&state_dir)?;

        let state_path = state_dir.join("state.json");
        let state: ProjectState = read_json_or_default(&state_path)?;
        let state = Arc::new(RwLock::new(state));

        Ok(Self {
            task_service: TaskService::new(Arc::clone(&state)),
            requirement_service: RequirementService::new(Arc::clone(&state)),
            state: Arc::clone(&state),
            state_path,
        })
    }

    pub fn in_memory() -> Self {
        let state = Arc::new(RwLock::new(ProjectState::default()));
        Self {
            task_service: TaskService::new(Arc::clone(&state)),
            requirement_service: RequirementService::new(Arc::clone(&state)),
            state: Arc::clone(&state),
            state_path: PathBuf::from("/dev/null"),
        }
    }

    pub fn tasks(&self) -> &TaskService {
        &self.task_service
    }

    pub fn requirements(&self) -> &RequirementService {
        &self.requirement_service
    }

    pub async fn persist(&self) -> Result<()> {
        let state = self.state.read().await;
        write_json_atomic(&self.state_path, &*state)?;
        Ok(())
    }
}
