mod tools;

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use std::path::PathBuf;
use tools::ProjectsMcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let project_root = std::env::var("AO_PROJECTS_ROOT")
        .map(PathBuf::from)
        .or_else(|_| std::env::current_dir())
        .unwrap_or_else(|_| PathBuf::from("."));

    tracing::info!("ao-projects-mcp starting for {}", project_root.display());

    let server = ProjectsMcpServer::new(&project_root)?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
