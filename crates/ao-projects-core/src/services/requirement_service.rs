use anyhow::Result;
use ao_projects_protocol::*;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::ProjectState;

pub struct RequirementService {
    state: Arc<RwLock<ProjectState>>,
}

impl RequirementService {
    pub fn new(state: Arc<RwLock<ProjectState>>) -> Self {
        Self { state }
    }

    pub async fn list(&self, filter: Option<RequirementFilter>) -> Result<Vec<RequirementItem>> {
        let state = self.state.read().await;
        let mut reqs: Vec<RequirementItem> = state.requirements.values().cloned().collect();
        if let Some(f) = filter {
            reqs.retain(|r| requirement_matches_filter(r, &f));
        }
        reqs.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(reqs)
    }

    pub async fn get(&self, id: &str) -> Result<RequirementItem> {
        let state = self.state.read().await;
        state
            .requirements
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("requirement not found: {}", id))
    }

    pub async fn create(&self, input: RequirementCreateInput) -> Result<RequirementItem> {
        let mut state = self.state.write().await;
        let id = state.next_requirement_id();
        let now = Utc::now();
        let req = RequirementItem {
            id: id.clone(),
            title: input.title,
            description: input.description.unwrap_or_default(),
            body: None,
            legacy_id: None,
            category: input.category,
            requirement_type: input.requirement_type.map(Some).unwrap_or(None),
            acceptance_criteria: input.acceptance_criteria,
            priority: input.priority.unwrap_or_default(),
            status: RequirementStatus::Draft,
            source: input.source.unwrap_or_else(|| "manual".to_string()),
            tags: Vec::new(),
            links: RequirementLinks::default(),
            comments: Vec::new(),
            relative_path: None,
            linked_task_ids: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        state.requirements.insert(id.clone(), req.clone());
        state.dirty_requirements.insert(id);
        Ok(req)
    }

    pub async fn update(&self, id: &str, input: RequirementUpdateInput) -> Result<RequirementItem> {
        let mut state = self.state.write().await;
        let req = state
            .requirements
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("requirement not found: {}", id))?;

        if let Some(title) = input.title {
            req.title = title;
        }
        if let Some(desc) = input.description {
            req.description = desc;
        }
        if let Some(priority) = input.priority {
            req.priority = priority;
        }
        if let Some(status) = input.status {
            req.status = status;
        }
        if let Some(category) = input.category {
            req.category = Some(category);
        }
        if let Some(req_type) = input.requirement_type {
            req.requirement_type = Some(req_type);
        }
        if let Some(criteria) = input.acceptance_criteria {
            if input.replace_acceptance_criteria {
                req.acceptance_criteria = criteria;
            } else {
                req.acceptance_criteria.extend(criteria);
            }
        }
        if let Some(task_id) = input.linked_task_id
            && !req.linked_task_ids.contains(&task_id)
        {
            req.linked_task_ids.push(task_id);
        }
        req.updated_at = Utc::now();

        let req = req.clone();
        state.dirty_requirements.insert(id.to_string());
        Ok(req)
    }

    pub async fn upsert(&self, req: RequirementItem) -> Result<RequirementItem> {
        let mut state = self.state.write().await;
        let id = req.id.clone();
        state.requirements.insert(id.clone(), req.clone());
        state.dirty_requirements.insert(id);
        Ok(req)
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        let mut state = self.state.write().await;
        state
            .requirements
            .remove(id)
            .ok_or_else(|| anyhow::anyhow!("requirement not found: {}", id))?;
        state.all_requirements_dirty = true;
        Ok(())
    }

    pub async fn refine(&self, id: &str) -> Result<RequirementItem> {
        let mut state = self.state.write().await;
        let req = state
            .requirements
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("requirement not found: {}", id))?;

        req.status = RequirementStatus::Refined;
        if req.acceptance_criteria.is_empty() {
            req.acceptance_criteria
                .push("Acceptance criteria to be defined".to_string());
        }
        req.updated_at = Utc::now();

        let req = req.clone();
        state.dirty_requirements.insert(id.to_string());
        Ok(req)
    }
}

fn requirement_matches_filter(req: &RequirementItem, filter: &RequirementFilter) -> bool {
    if let Some(ref s) = filter.status
        && &req.status != s
    {
        return false;
    }
    if let Some(ref p) = filter.priority
        && &req.priority != p
    {
        return false;
    }
    if let Some(ref c) = filter.category
        && req.category.as_deref() != Some(c.as_str())
    {
        return false;
    }
    if let Some(ref t) = filter.requirement_type
        && req.requirement_type.as_ref() != Some(t)
    {
        return false;
    }
    if let Some(ref task_id) = filter.linked_task_id
        && !req.linked_task_ids.contains(task_id)
    {
        return false;
    }
    if let Some(ref tags) = filter.tags
        && !tags.is_empty()
        && !tags.iter().any(|t| req.tags.contains(t))
    {
        return false;
    }
    if let Some(ref search) = filter.search_text {
        let s = search.to_lowercase();
        let matches = req.title.to_lowercase().contains(&s)
            || req.description.to_lowercase().contains(&s)
            || req.id.to_lowercase().contains(&s);
        if !matches {
            return false;
        }
    }
    true
}
