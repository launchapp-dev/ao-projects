use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{RequirementService, TaskService};
use crate::state::ProjectState;
use ao_projects_protocol::{OrchestratorTask, TaskCreateInput};
use ao_projects_store::{read_json_or_default, scoped_state_root, write_json_atomic};

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

    pub async fn create_task_linked(&self, input: TaskCreateInput) -> Result<OrchestratorTask> {
        let linked_reqs = input.linked_requirements.clone();
        let task = self.task_service.create(input).await?;

        if !linked_reqs.is_empty() {
            let mut state = self.state.write().await;
            for req_id in &linked_reqs {
                if let Some(req) = state.requirements.get_mut(req_id)
                    && !req.linked_task_ids.contains(&task.id)
                {
                    req.linked_task_ids.push(task.id.clone());
                    req.updated_at = chrono::Utc::now();
                    state.dirty_requirements.insert(req_id.clone());
                }
            }
        }
        Ok(task)
    }

    pub async fn persist(&self) -> Result<()> {
        let state = self.state.read().await;
        write_json_atomic(&self.state_path, &*state)?;
        Ok(())
    }
}
