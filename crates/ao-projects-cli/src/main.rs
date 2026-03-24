use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use ao_projects_core::{ProjectHub, SyncConfig, SyncClient};

#[derive(Parser)]
#[command(name = "ao-projects", about = "Task and requirement management for AI-driven pipelines")]
struct Cli {
    #[arg(long, env = "AO_PROJECTS_ROOT")]
    project_root: Option<PathBuf>,

    #[arg(long, default_value = "false")]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
    #[command(alias = "req")]
    Requirements {
        #[command(subcommand)]
        command: RequirementsCommand,
    },
    Sync {
        #[command(subcommand)]
        command: SyncCommand,
    },
}

#[derive(Subcommand)]
enum TaskCommand {
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        search: Option<String>,
        #[arg(long, default_value = "50")]
        limit: usize,
    },
    Get {
        #[arg(long)]
        id: String,
    },
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        task_type: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        tag: Vec<String>,
    },
    Update {
        #[arg(long)]
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        priority: Option<String>,
    },
    Status {
        #[arg(long)]
        id: String,
        #[arg(long)]
        status: String,
    },
    Delete {
        #[arg(long)]
        id: String,
    },
    Stats,
    #[command(name = "checklist-add")]
    ChecklistAdd {
        #[arg(long)]
        id: String,
        #[arg(long)]
        description: String,
    },
    #[command(name = "checklist-update")]
    ChecklistUpdate {
        #[arg(long)]
        id: String,
        #[arg(long)]
        item_id: String,
        #[arg(long)]
        completed: bool,
    },
}

#[derive(Subcommand)]
enum RequirementsCommand {
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        search: Option<String>,
        #[arg(long, default_value = "50")]
        limit: usize,
    },
    Get {
        #[arg(long)]
        id: String,
    },
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        acceptance_criterion: Vec<String>,
    },
    Update {
        #[arg(long)]
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        category: Option<String>,
    },
    Delete {
        #[arg(long)]
        id: String,
    },
    Refine {
        #[arg(long)]
        id: String,
    },
}

#[derive(Subcommand)]
enum SyncCommand {
    Setup {
        #[arg(long)]
        server: String,
        #[arg(long)]
        token: String,
    },
    Push,
    Pull,
    Status,
    Link {
        #[arg(long)]
        project_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let project_root = cli.project_root
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    let hub = ProjectHub::load(&project_root)?;

    match cli.command {
        Command::Task { command } => handle_task(command, &hub, cli.json).await?,
        Command::Requirements { command } => handle_requirements(command, &hub, cli.json).await?,
        Command::Sync { command } => handle_sync(command, &hub, &project_root, cli.json).await?,
    }

    hub.persist().await?;
    Ok(())
}

async fn handle_task(cmd: TaskCommand, hub: &ProjectHub, json: bool) -> Result<()> {
    match cmd {
        TaskCommand::List { status: _, priority: _, search, limit: _ } => {
            let filter = search.map(|s| ao_projects_protocol::TaskFilter {
                search_text: Some(s),
                ..Default::default()
            });
            let tasks = hub.tasks().list(filter).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&tasks)?);
            } else {
                for t in &tasks {
                    println!("{} [{}] {:?} — {}", t.id, format!("{:?}", t.status).to_lowercase(), t.priority, t.title);
                }
                println!("\n{} tasks", tasks.len());
            }
        }
        TaskCommand::Get { id } => {
            let task = hub.tasks().get(&id).await?;
            println!("{}", serde_json::to_string_pretty(&task)?);
        }
        TaskCommand::Create { title, description, task_type: _, priority: _, tag } => {
            let input = ao_projects_protocol::TaskCreateInput {
                title,
                description,
                tags: tag,
                ..Default::default()
            };
            let task = hub.tasks().create(input).await?;
            println!("Created {}: {}", task.id, task.title);
        }
        TaskCommand::Stats => {
            let stats = hub.tasks().statistics().await?;
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
        TaskCommand::Status { id, status: _ } => {
            let task = hub.tasks().set_status(&id, ao_projects_protocol::TaskStatus::Ready).await?;
            println!("Updated {}: {:?}", task.id, task.status);
        }
        TaskCommand::Delete { id } => {
            hub.tasks().delete(&id).await?;
            println!("Deleted {}", id);
        }
        TaskCommand::Update { id, title, description, priority: _ } => {
            let input = ao_projects_protocol::TaskUpdateInput {
                title,
                description,
                ..Default::default()
            };
            let task = hub.tasks().update(&id, input).await?;
            println!("Updated {}: {}", task.id, task.title);
        }
        TaskCommand::ChecklistAdd { id, description } => {
            let task = hub.tasks().add_checklist_item(&id, description).await?;
            println!("Added checklist item to {}", task.id);
        }
        TaskCommand::ChecklistUpdate { id, item_id, completed } => {
            let task = hub.tasks().update_checklist_item(&id, &item_id, completed).await?;
            println!("Updated checklist on {}", task.id);
        }
    }
    Ok(())
}

async fn handle_requirements(cmd: RequirementsCommand, hub: &ProjectHub, json: bool) -> Result<()> {
    match cmd {
        RequirementsCommand::List { status: _, priority: _, category: _, search, limit: _ } => {
            let filter = search.map(|s| ao_projects_protocol::RequirementFilter {
                search_text: Some(s),
                ..Default::default()
            });
            let reqs = hub.requirements().list(filter).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&reqs)?);
            } else {
                for r in &reqs {
                    println!("{} [{}] {:?} — {}", r.id, format!("{:?}", r.status).to_lowercase(), r.priority, r.title);
                }
                println!("\n{} requirements", reqs.len());
            }
        }
        RequirementsCommand::Get { id } => {
            let req = hub.requirements().get(&id).await?;
            println!("{}", serde_json::to_string_pretty(&req)?);
        }
        RequirementsCommand::Create { title, description, priority: _, category, acceptance_criterion } => {
            let input = ao_projects_protocol::RequirementCreateInput {
                title,
                description,
                category,
                acceptance_criteria: acceptance_criterion,
                ..Default::default()
            };
            let req = hub.requirements().create(input).await?;
            println!("Created {}: {}", req.id, req.title);
        }
        RequirementsCommand::Update { id, title, description, priority: _, status: _, category: _ } => {
            let input = ao_projects_protocol::RequirementUpdateInput {
                title,
                description,
                ..Default::default()
            };
            let req = hub.requirements().update(&id, input).await?;
            println!("Updated {}: {}", req.id, req.title);
        }
        RequirementsCommand::Delete { id } => {
            hub.requirements().delete(&id).await?;
            println!("Deleted {}", id);
        }
        RequirementsCommand::Refine { id } => {
            let req = hub.requirements().refine(&id).await?;
            println!("Refined {}: {:?}", req.id, req.status);
        }
    }
    Ok(())
}

async fn handle_sync(cmd: SyncCommand, hub: &ProjectHub, project_root: &Path, json: bool) -> Result<()> {
    let root_str = project_root.to_string_lossy();
    match cmd {
        SyncCommand::Setup { server, token } => {
            let mut config = SyncConfig::load_global();
            config.server = Some(server.clone());
            config.token = Some(token);
            config.save_global()?;
            if json {
                println!("{}", serde_json::json!({"configured": true, "server": server}));
            } else {
                println!("Sync server configured: {}", server);
                println!("Link project with: ao-projects sync link --project-id <id>");
            }
        }
        SyncCommand::Link { project_id } => {
            let mut config = SyncConfig::load_for_project(&root_str);
            config.project_id = Some(project_id.clone());
            config.save_for_project(&root_str)?;
            if json {
                println!("{}", serde_json::json!({"linked": true, "project_id": project_id}));
            } else {
                println!("Linked to project: {}", project_id);
            }
        }
        SyncCommand::Push => {
            let config = SyncConfig::load_for_project(&root_str);
            let client = SyncClient::new(config)?;
            let result = client.push(hub).await?;
            if json {
                println!("{}", serde_json::json!({
                    "tasks_sent": result.tasks_sent,
                    "requirements_sent": result.requirements_sent,
                    "conflicts": result.conflicts,
                    "server_time": result.server_time,
                }));
            } else {
                println!("Pushed {} tasks, {} requirements", result.tasks_sent, result.requirements_sent);
                if result.conflicts > 0 {
                    println!("Conflicts: {}", result.conflicts);
                }
            }
        }
        SyncCommand::Pull => {
            let config = SyncConfig::load_for_project(&root_str);
            let client = SyncClient::new(config)?;
            let result = client.pull(hub).await?;
            if json {
                println!("{}", serde_json::json!({
                    "tasks_received": result.tasks_received,
                    "requirements_received": result.requirements_received,
                    "server_time": result.server_time,
                }));
            } else {
                println!("Pulled {} tasks, {} requirements", result.tasks_received, result.requirements_received);
            }
        }
        SyncCommand::Status => {
            let config = SyncConfig::load_for_project(&root_str);
            if json {
                println!("{}", serde_json::json!({
                    "configured": config.is_configured(),
                    "server": config.server,
                    "project_id": config.project_id,
                    "last_synced_at": config.last_synced_at,
                }));
            } else {
                println!("Configured: {}", config.is_configured());
                println!("Server: {}", config.server.as_deref().unwrap_or("(not set)"));
                println!("Project: {}", config.project_id.as_deref().unwrap_or("(not linked)"));
                println!("Last sync: {}", config.last_synced_at.as_deref().unwrap_or("never"));
            }
        }
    }
    Ok(())
}

use std::path::Path;
