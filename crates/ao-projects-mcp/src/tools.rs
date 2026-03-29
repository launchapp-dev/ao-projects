use ao_projects_core::ProjectHub;
use ao_projects_protocol::*;
use rmcp::schemars::{self, JsonSchema};
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

pub struct ProjectsMcpServer {
    hub: Arc<ProjectHub>,
    tool_router: ToolRouter<Self>,
}

impl ProjectsMcpServer {
    pub fn new(project_root: &Path) -> anyhow::Result<Self> {
        let hub = Arc::new(ProjectHub::load(project_root)?);
        let tool_router = Self::task_tools() + Self::requirement_tools();
        Ok(Self { hub, tool_router })
    }
}

fn ok_json<T: Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let text =
        serde_json::to_string_pretty(value).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"));
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}

fn err(msg: String) -> McpError {
    McpError::new(rmcp::model::ErrorCode::INTERNAL_ERROR, msg, None)
}

// --- Input types ---

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
struct TaskListInput {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    search: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
struct IdInput {
    id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
struct TaskCreateMcpInput {
    title: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    task_type: Option<String>,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    linked_requirements: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
struct TaskStatusMcpInput {
    id: String,
    status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
struct TaskUpdateMcpInput {
    id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    priority: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
struct ChecklistAddMcpInput {
    id: String,
    description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
struct ChecklistUpdateMcpInput {
    id: String,
    item_id: String,
    completed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
struct ReqListInput {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    search: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
struct ReqCreateMcpInput {
    title: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    acceptance_criteria: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
struct ReqUpdateMcpInput {
    id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    category: Option<String>,
}

fn schema_for<T: JsonSchema + std::any::Any>() -> std::sync::Arc<rmcp::model::JsonObject> {
    rmcp::handler::server::common::schema_for_type::<T>()
}

// --- Task Tools ---

#[tool_router(router = task_tools, vis = "pub(crate)")]
impl ProjectsMcpServer {
    #[tool(
        name = "projects.task.list",
        description = "List tasks with optional filters (status, priority, search). Returns tasks sorted by priority.",
        input_schema = schema_for::<TaskListInput>()
    )]
    async fn task_list(
        &self,
        params: Parameters<TaskListInput>,
    ) -> Result<CallToolResult, McpError> {
        let input = params.0;
        let filter = if input.status.is_some() || input.priority.is_some() || input.search.is_some()
        {
            Some(TaskFilter {
                search_text: input.search,
                ..Default::default()
            })
        } else {
            None
        };
        let tasks = self
            .hub
            .tasks()
            .list(filter)
            .await
            .map_err(|e| err(e.to_string()))?;
        let limited: Vec<_> = tasks.into_iter().take(input.limit.unwrap_or(50)).collect();
        ok_json(&limited)
    }

    #[tool(
        name = "projects.task.get",
        description = "Get a task by ID. Returns full task details including checklist, dependencies, and metadata.",
        input_schema = schema_for::<IdInput>()
    )]
    async fn task_get(&self, params: Parameters<IdInput>) -> Result<CallToolResult, McpError> {
        let task = self
            .hub
            .tasks()
            .get(&params.0.id)
            .await
            .map_err(|e| err(e.to_string()))?;
        ok_json(&task)
    }

    #[tool(
        name = "projects.task.create",
        description = "Create a new task. Returns the created task with generated ID.",
        input_schema = schema_for::<TaskCreateMcpInput>()
    )]
    async fn task_create(
        &self,
        params: Parameters<TaskCreateMcpInput>,
    ) -> Result<CallToolResult, McpError> {
        let input = params.0;
        let create = TaskCreateInput {
            title: input.title,
            description: input.description.unwrap_or_default(),
            tags: input.tags,
            linked_requirements: input.linked_requirements,
            ..Default::default()
        };
        let task = self
            .hub
            .tasks()
            .create(create)
            .await
            .map_err(|e| err(e.to_string()))?;
        self.hub.persist().await.map_err(|e| err(e.to_string()))?;
        ok_json(&task)
    }

    #[tool(
        name = "projects.task.update",
        description = "Update task fields (title, description, priority).",
        input_schema = schema_for::<TaskUpdateMcpInput>()
    )]
    async fn task_update(
        &self,
        params: Parameters<TaskUpdateMcpInput>,
    ) -> Result<CallToolResult, McpError> {
        let input = params.0;
        let update = TaskUpdateInput {
            title: input.title,
            description: input.description,
            ..Default::default()
        };
        let task = self
            .hub
            .tasks()
            .update(&input.id, update)
            .await
            .map_err(|e| err(e.to_string()))?;
        self.hub.persist().await.map_err(|e| err(e.to_string()))?;
        ok_json(&task)
    }

    #[tool(
        name = "projects.task.status",
        description = "Set task status (backlog, ready, in_progress, blocked, on_hold, done, cancelled).",
        input_schema = schema_for::<TaskStatusMcpInput>()
    )]
    async fn task_status(
        &self,
        params: Parameters<TaskStatusMcpInput>,
    ) -> Result<CallToolResult, McpError> {
        let input = params.0;
        let status: TaskStatus = serde_json::from_value(serde_json::Value::String(input.status))
            .map_err(|e| err(format!("invalid status: {e}")))?;
        let task = self
            .hub
            .tasks()
            .set_status(&input.id, status)
            .await
            .map_err(|e| err(e.to_string()))?;
        self.hub.persist().await.map_err(|e| err(e.to_string()))?;
        ok_json(&task)
    }

    #[tool(
        name = "projects.task.delete",
        description = "Delete a task by ID.",
        input_schema = schema_for::<IdInput>()
    )]
    async fn task_delete(&self, params: Parameters<IdInput>) -> Result<CallToolResult, McpError> {
        self.hub
            .tasks()
            .delete(&params.0.id)
            .await
            .map_err(|e| err(e.to_string()))?;
        self.hub.persist().await.map_err(|e| err(e.to_string()))?;
        ok_json(&serde_json::json!({"deleted": true, "id": params.0.id}))
    }

    #[tool(
        name = "projects.task.stats",
        description = "Get aggregate task statistics (counts by status, priority, type).",
        input_schema = schema_for::<serde_json::Value>()
    )]
    async fn task_stats(
        &self,
        _params: Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, McpError> {
        let stats = self
            .hub
            .tasks()
            .statistics()
            .await
            .map_err(|e| err(e.to_string()))?;
        ok_json(&stats)
    }

    #[tool(
        name = "projects.task.checklist-add",
        description = "Add a checklist item to a task.",
        input_schema = schema_for::<ChecklistAddMcpInput>()
    )]
    async fn task_checklist_add(
        &self,
        params: Parameters<ChecklistAddMcpInput>,
    ) -> Result<CallToolResult, McpError> {
        let input = params.0;
        let task = self
            .hub
            .tasks()
            .add_checklist_item(&input.id, input.description)
            .await
            .map_err(|e| err(e.to_string()))?;
        self.hub.persist().await.map_err(|e| err(e.to_string()))?;
        ok_json(&task)
    }

    #[tool(
        name = "projects.task.checklist-update",
        description = "Update a checklist item completion status.",
        input_schema = schema_for::<ChecklistUpdateMcpInput>()
    )]
    async fn task_checklist_update(
        &self,
        params: Parameters<ChecklistUpdateMcpInput>,
    ) -> Result<CallToolResult, McpError> {
        let input = params.0;
        let task = self
            .hub
            .tasks()
            .update_checklist_item(&input.id, &input.item_id, input.completed)
            .await
            .map_err(|e| err(e.to_string()))?;
        self.hub.persist().await.map_err(|e| err(e.to_string()))?;
        ok_json(&task)
    }
}

// --- Requirement Tools ---

#[tool_router(router = requirement_tools, vis = "pub(crate)")]
impl ProjectsMcpServer {
    #[tool(
        name = "projects.req.list",
        description = "List requirements with optional filters (status, priority, category, search).",
        input_schema = schema_for::<ReqListInput>()
    )]
    async fn req_list(&self, params: Parameters<ReqListInput>) -> Result<CallToolResult, McpError> {
        let input = params.0;
        let filter = if input.status.is_some()
            || input.priority.is_some()
            || input.category.is_some()
            || input.search.is_some()
        {
            Some(RequirementFilter {
                search_text: input.search,
                category: input.category,
                ..Default::default()
            })
        } else {
            None
        };
        let reqs = self
            .hub
            .requirements()
            .list(filter)
            .await
            .map_err(|e| err(e.to_string()))?;
        let limited: Vec<_> = reqs.into_iter().take(input.limit.unwrap_or(50)).collect();
        ok_json(&limited)
    }

    #[tool(
        name = "projects.req.get",
        description = "Get a requirement by ID. Returns full details including acceptance criteria and linked tasks.",
        input_schema = schema_for::<IdInput>()
    )]
    async fn req_get(&self, params: Parameters<IdInput>) -> Result<CallToolResult, McpError> {
        let req = self
            .hub
            .requirements()
            .get(&params.0.id)
            .await
            .map_err(|e| err(e.to_string()))?;
        ok_json(&req)
    }

    #[tool(
        name = "projects.req.create",
        description = "Create a new requirement. Returns the created requirement with generated ID.",
        input_schema = schema_for::<ReqCreateMcpInput>()
    )]
    async fn req_create(
        &self,
        params: Parameters<ReqCreateMcpInput>,
    ) -> Result<CallToolResult, McpError> {
        let input = params.0;
        let create = RequirementCreateInput {
            title: input.title,
            description: input.description,
            category: input.category,
            acceptance_criteria: input.acceptance_criteria,
            ..Default::default()
        };
        let req = self
            .hub
            .requirements()
            .create(create)
            .await
            .map_err(|e| err(e.to_string()))?;
        self.hub.persist().await.map_err(|e| err(e.to_string()))?;
        ok_json(&req)
    }

    #[tool(
        name = "projects.req.update",
        description = "Update requirement fields (title, description, priority, status, category).",
        input_schema = schema_for::<ReqUpdateMcpInput>()
    )]
    async fn req_update(
        &self,
        params: Parameters<ReqUpdateMcpInput>,
    ) -> Result<CallToolResult, McpError> {
        let input = params.0;
        let update = RequirementUpdateInput {
            title: input.title,
            description: input.description,
            category: input.category,
            ..Default::default()
        };
        let req = self
            .hub
            .requirements()
            .update(&input.id, update)
            .await
            .map_err(|e| err(e.to_string()))?;
        self.hub.persist().await.map_err(|e| err(e.to_string()))?;
        ok_json(&req)
    }

    #[tool(
        name = "projects.req.delete",
        description = "Delete a requirement by ID.",
        input_schema = schema_for::<IdInput>()
    )]
    async fn req_delete(&self, params: Parameters<IdInput>) -> Result<CallToolResult, McpError> {
        self.hub
            .requirements()
            .delete(&params.0.id)
            .await
            .map_err(|e| err(e.to_string()))?;
        self.hub.persist().await.map_err(|e| err(e.to_string()))?;
        ok_json(&serde_json::json!({"deleted": true, "id": params.0.id}))
    }

    #[tool(
        name = "projects.req.refine",
        description = "Refine a draft requirement: set status to refined, ensure acceptance criteria.",
        input_schema = schema_for::<IdInput>()
    )]
    async fn req_refine(&self, params: Parameters<IdInput>) -> Result<CallToolResult, McpError> {
        let req = self
            .hub
            .requirements()
            .refine(&params.0.id)
            .await
            .map_err(|e| err(e.to_string()))?;
        self.hub.persist().await.map_err(|e| err(e.to_string()))?;
        ok_json(&req)
    }
}

// --- ServerHandler ---

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ProjectsMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Task and requirement management tools for AI-driven development pipelines.",
        )
    }
}
