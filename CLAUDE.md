# ao-projects - Coding Agent Guide

## What is this?

Standalone task and requirements manager extracted from AO CLI.
Wire-compatible with AO's `OrchestratorTask` and `RequirementItem` types.

## Workspace Map

- `crates/ao-projects-protocol` — Wire types (Task, Requirement, Priority, Status, Filter)
- `crates/ao-projects-core` — Service layer (TaskService, RequirementService, ProjectHub, SyncClient)
- `crates/ao-projects-store` — Atomic JSON I/O, scoped state paths
- `crates/ao-projects-cli` — CLI binary (`ao-projects task/req/sync`)
- `crates/ao-projects-mcp` — MCP server (15 tools via rmcp 1.2)

## Build & Test

```bash
cargo check
cargo test
cargo build --release
```

## Working Rules

- Types in protocol crate MUST stay wire-compatible with AO CLI's `OrchestratorTask` and `RequirementItem`
- Serde rename strategies: TaskStatus=kebab-case, TaskType=lowercase, Priority=lowercase, RequirementStatus=kebab-case
- All enums support FromStr with aliases (e.g., "in-progress", "in_progress", "todo"→backlog)
- MCP server calls service methods directly (no subprocess)
- Tests use `ProjectHub::in_memory()` for fast isolated testing
