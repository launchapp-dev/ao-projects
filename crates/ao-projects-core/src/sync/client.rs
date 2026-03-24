use anyhow::{Context, Result};
use ao_projects_protocol::{Task, Requirement};
use serde::{Deserialize, Serialize};

use super::SyncConfig;
use crate::ProjectHub;

pub struct SyncClient {
    config: SyncConfig,
    http: reqwest::Client,
}

#[derive(Serialize)]
struct SyncRequest {
    tasks: Vec<Task>,
    requirements: Vec<Requirement>,
    since: Option<String>,
}

#[derive(Deserialize)]
pub struct SyncResponse {
    pub tasks: Vec<Task>,
    pub requirements: Vec<Requirement>,
    pub conflicts: Vec<SyncConflict>,
    pub server_time: String,
}

#[derive(Deserialize)]
pub struct SyncConflict {
    pub r#type: String,
    pub id: String,
    pub reason: String,
}

#[derive(Deserialize)]
struct ProjectResponse {
    project: ProjectInfo,
}

#[derive(Deserialize)]
struct ProjectInfo {
    id: String,
    name: String,
}

#[derive(Debug)]
pub struct PushResult {
    pub tasks_sent: usize,
    pub requirements_sent: usize,
    pub conflicts: usize,
    pub server_time: String,
}

#[derive(Debug)]
pub struct PullResult {
    pub tasks_received: usize,
    pub requirements_received: usize,
    pub server_time: String,
}

#[derive(Debug)]
pub struct LinkResult {
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub auto_linked: bool,
}

impl SyncClient {
    pub fn new(config: SyncConfig) -> Result<Self> {
        let token = config.bearer_token()?;
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))?,
        );
        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { config, http })
    }

    pub async fn push(&self, hub: &ProjectHub) -> Result<PushResult> {
        let project_id = self.config.project_id.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No project linked. Run: ao-projects sync link --project-id <id>"))?;
        let server = self.config.server_url()?;

        let tasks = hub.tasks().list(None).await?;
        let requirements = hub.requirements().list(None).await?;
        let tasks_count = tasks.len();
        let reqs_count = requirements.len();

        let resp = self.http
            .post(format!("{}/api/projects/{}/sync", server.trim_end_matches('/'), project_id))
            .json(&SyncRequest {
                tasks,
                requirements,
                since: self.config.last_synced_at.clone(),
            })
            .send()
            .await
            .context("Failed to connect to sync server")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Sync push failed ({status}): {body}");
        }

        let sync_resp: SyncResponse = resp.json().await.context("Failed to parse sync response")?;

        Ok(PushResult {
            tasks_sent: tasks_count,
            requirements_sent: reqs_count,
            conflicts: sync_resp.conflicts.len(),
            server_time: sync_resp.server_time,
        })
    }

    pub async fn pull(&self, hub: &ProjectHub) -> Result<PullResult> {
        let project_id = self.config.project_id.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No project linked. Run: ao-projects sync link --project-id <id>"))?;
        let server = self.config.server_url()?;

        let resp = self.http
            .post(format!("{}/api/projects/{}/sync", server.trim_end_matches('/'), project_id))
            .json(&SyncRequest {
                tasks: vec![],
                requirements: vec![],
                since: self.config.last_synced_at.clone(),
            })
            .send()
            .await
            .context("Failed to connect to sync server")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Sync pull failed ({status}): {body}");
        }

        let sync_resp: SyncResponse = resp.json().await.context("Failed to parse sync response")?;
        let task_count = sync_resp.tasks.len();
        let req_count = sync_resp.requirements.len();

        // TODO: Apply pulled tasks and requirements to local state
        // For now, this requires direct state mutation via hub

        Ok(PullResult {
            tasks_received: task_count,
            requirements_received: req_count,
            server_time: sync_resp.server_time,
        })
    }

    pub async fn auto_link(&self, git_origin_url: &str) -> Result<LinkResult> {
        let server = self.config.server_url()?;
        let encoded = urlencoding(git_origin_url);
        let resp = self.http
            .get(format!("{}/api/projects/by-repo?url={}", server.trim_end_matches('/'), encoded))
            .send()
            .await;

        if let Ok(resp) = resp {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<ProjectResponse>().await {
                    return Ok(LinkResult {
                        project_id: Some(body.project.id),
                        project_name: Some(body.project.name),
                        auto_linked: true,
                    });
                }
            }
        }

        Ok(LinkResult {
            project_id: None,
            project_name: None,
            auto_linked: false,
        })
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}
