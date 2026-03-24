use ao_projects_core::ProjectHub;
use ao_projects_protocol::*;

#[tokio::test]
async fn create_and_get_task() {
    let hub = ProjectHub::in_memory();
    let task = hub.tasks().create(TaskCreateInput {
        title: "Test task".into(),
        description: Some("A description".into()),
        ..Default::default()
    }).await.unwrap();

    assert_eq!(task.id, "TASK-001");
    assert_eq!(task.title, "Test task");
    assert_eq!(task.status, TaskStatus::Backlog);
    assert_eq!(task.priority, Priority::Medium);

    let fetched = hub.tasks().get("TASK-001").await.unwrap();
    assert_eq!(fetched.title, "Test task");
}

#[tokio::test]
async fn sequential_ids() {
    let hub = ProjectHub::in_memory();
    let t1 = hub.tasks().create(TaskCreateInput { title: "First".into(), ..Default::default() }).await.unwrap();
    let t2 = hub.tasks().create(TaskCreateInput { title: "Second".into(), ..Default::default() }).await.unwrap();
    let t3 = hub.tasks().create(TaskCreateInput { title: "Third".into(), ..Default::default() }).await.unwrap();

    assert_eq!(t1.id, "TASK-001");
    assert_eq!(t2.id, "TASK-002");
    assert_eq!(t3.id, "TASK-003");
}

#[tokio::test]
async fn status_transitions() {
    let hub = ProjectHub::in_memory();
    hub.tasks().create(TaskCreateInput { title: "Task".into(), ..Default::default() }).await.unwrap();

    let task = hub.tasks().set_status("TASK-001", TaskStatus::Ready).await.unwrap();
    assert_eq!(task.status, TaskStatus::Ready);
    assert!(!task.paused);

    let task = hub.tasks().set_status("TASK-001", TaskStatus::InProgress).await.unwrap();
    assert_eq!(task.status, TaskStatus::InProgress);
    assert!(task.metadata.started_at.is_some());

    let task = hub.tasks().set_status("TASK-001", TaskStatus::Done).await.unwrap();
    assert_eq!(task.status, TaskStatus::Done);
    assert!(task.metadata.completed_at.is_some());
}

#[tokio::test]
async fn blocked_sets_paused() {
    let hub = ProjectHub::in_memory();
    hub.tasks().create(TaskCreateInput { title: "Task".into(), ..Default::default() }).await.unwrap();

    let task = hub.tasks().set_status("TASK-001", TaskStatus::Blocked).await.unwrap();
    assert!(task.paused);
    assert!(task.blocked_at.is_some());

    let task = hub.tasks().set_status("TASK-001", TaskStatus::Ready).await.unwrap();
    assert!(!task.paused);
    assert!(task.blocked_at.is_none());
}

#[tokio::test]
async fn filter_by_status() {
    let hub = ProjectHub::in_memory();
    hub.tasks().create(TaskCreateInput { title: "Ready one".into(), ..Default::default() }).await.unwrap();
    hub.tasks().create(TaskCreateInput { title: "Ready two".into(), ..Default::default() }).await.unwrap();
    hub.tasks().set_status("TASK-001", TaskStatus::Ready).await.unwrap();

    let ready = hub.tasks().list(Some(TaskFilter {
        status: Some(TaskStatus::Ready),
        ..Default::default()
    })).await.unwrap();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "TASK-001");
}

#[tokio::test]
async fn search_text() {
    let hub = ProjectHub::in_memory();
    hub.tasks().create(TaskCreateInput { title: "Fix login bug".into(), ..Default::default() }).await.unwrap();
    hub.tasks().create(TaskCreateInput { title: "Add dashboard".into(), ..Default::default() }).await.unwrap();

    let found = hub.tasks().list(Some(TaskFilter {
        search_text: Some("login".into()),
        ..Default::default()
    })).await.unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].title, "Fix login bug");
}

#[tokio::test]
async fn checklist_operations() {
    let hub = ProjectHub::in_memory();
    hub.tasks().create(TaskCreateInput { title: "Task".into(), ..Default::default() }).await.unwrap();

    let task = hub.tasks().add_checklist_item("TASK-001", "Step 1".into()).await.unwrap();
    assert_eq!(task.checklist.len(), 1);
    assert!(!task.checklist[0].completed);

    let item_id = task.checklist[0].id.clone();
    let task = hub.tasks().update_checklist_item("TASK-001", &item_id, true).await.unwrap();
    assert!(task.checklist[0].completed);
    assert!(task.checklist[0].completed_at.is_some());
}

#[tokio::test]
async fn dependency_management() {
    let hub = ProjectHub::in_memory();
    hub.tasks().create(TaskCreateInput { title: "Parent".into(), ..Default::default() }).await.unwrap();
    hub.tasks().create(TaskCreateInput { title: "Child".into(), ..Default::default() }).await.unwrap();

    let task = hub.tasks().add_dependency("TASK-002", "TASK-001", DependencyType::BlockedBy).await.unwrap();
    assert_eq!(task.dependencies.len(), 1);
    assert_eq!(task.dependencies[0].task_id, "TASK-001");

    let task = hub.tasks().remove_dependency("TASK-002", "TASK-001").await.unwrap();
    assert!(task.dependencies.is_empty());
}

#[tokio::test]
async fn delete_task() {
    let hub = ProjectHub::in_memory();
    hub.tasks().create(TaskCreateInput { title: "Doomed".into(), ..Default::default() }).await.unwrap();
    hub.tasks().delete("TASK-001").await.unwrap();
    assert!(hub.tasks().get("TASK-001").await.is_err());
}

#[tokio::test]
async fn statistics() {
    let hub = ProjectHub::in_memory();
    hub.tasks().create(TaskCreateInput { title: "A".into(), priority: Some(Priority::High), ..Default::default() }).await.unwrap();
    hub.tasks().create(TaskCreateInput { title: "B".into(), priority: Some(Priority::Low), ..Default::default() }).await.unwrap();
    hub.tasks().set_status("TASK-001", TaskStatus::InProgress).await.unwrap();

    let stats = hub.tasks().statistics().await.unwrap();
    assert_eq!(stats.total, 2);
    assert_eq!(stats.in_progress, 1);
}

#[tokio::test]
async fn priority_sorting() {
    let hub = ProjectHub::in_memory();
    hub.tasks().create(TaskCreateInput { title: "Low".into(), priority: Some(Priority::Low), ..Default::default() }).await.unwrap();
    hub.tasks().create(TaskCreateInput { title: "Critical".into(), priority: Some(Priority::Critical), ..Default::default() }).await.unwrap();
    hub.tasks().create(TaskCreateInput { title: "High".into(), priority: Some(Priority::High), ..Default::default() }).await.unwrap();

    let tasks = hub.tasks().list(None).await.unwrap();
    assert_eq!(tasks[0].title, "Critical");
    assert_eq!(tasks[1].title, "High");
    assert_eq!(tasks[2].title, "Low");
}
