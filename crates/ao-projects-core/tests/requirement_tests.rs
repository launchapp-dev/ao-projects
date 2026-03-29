use ao_projects_core::ProjectHub;
use ao_projects_protocol::*;

#[tokio::test]
async fn create_and_get_requirement() {
    let hub = ProjectHub::in_memory();
    let req = hub
        .requirements()
        .create(RequirementCreateInput {
            title: "Add login page".into(),
            description: Some("Users need to log in".into()),
            acceptance_criteria: vec!["Email/password form exists".into()],
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(req.id, "REQ-001");
    assert_eq!(req.status, RequirementStatus::Draft);
    assert_eq!(req.priority, RequirementPriority::Should);
    assert_eq!(req.acceptance_criteria.len(), 1);

    let fetched = hub.requirements().get("REQ-001").await.unwrap();
    assert_eq!(fetched.title, "Add login page");
}

#[tokio::test]
async fn refine_requirement() {
    let hub = ProjectHub::in_memory();
    hub.requirements()
        .create(RequirementCreateInput {
            title: "Feature X".into(),
            ..Default::default()
        })
        .await
        .unwrap();

    let req = hub.requirements().refine("REQ-001").await.unwrap();
    assert_eq!(req.status, RequirementStatus::Refined);
    assert!(!req.acceptance_criteria.is_empty());
}

#[tokio::test]
async fn filter_by_status() {
    let hub = ProjectHub::in_memory();
    hub.requirements()
        .create(RequirementCreateInput {
            title: "A".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    hub.requirements()
        .create(RequirementCreateInput {
            title: "B".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    hub.requirements().refine("REQ-001").await.unwrap();

    let refined = hub
        .requirements()
        .list(Some(RequirementFilter {
            status: Some(RequirementStatus::Refined),
            ..Default::default()
        }))
        .await
        .unwrap();
    assert_eq!(refined.len(), 1);
    assert_eq!(refined[0].id, "REQ-001");
}

#[tokio::test]
async fn filter_by_category() {
    let hub = ProjectHub::in_memory();
    hub.requirements()
        .create(RequirementCreateInput {
            title: "Security fix".into(),
            category: Some("security".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    hub.requirements()
        .create(RequirementCreateInput {
            title: "UI tweak".into(),
            category: Some("usability".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    let security = hub
        .requirements()
        .list(Some(RequirementFilter {
            category: Some("security".into()),
            ..Default::default()
        }))
        .await
        .unwrap();
    assert_eq!(security.len(), 1);
    assert_eq!(security[0].title, "Security fix");
}

#[tokio::test]
async fn update_requirement() {
    let hub = ProjectHub::in_memory();
    hub.requirements()
        .create(RequirementCreateInput {
            title: "Original".into(),
            ..Default::default()
        })
        .await
        .unwrap();

    let req = hub
        .requirements()
        .update(
            "REQ-001",
            RequirementUpdateInput {
                title: Some("Updated".into()),
                status: Some(RequirementStatus::Refined),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(req.title, "Updated");
    assert_eq!(req.status, RequirementStatus::Refined);
}

#[tokio::test]
async fn bidirectional_linking() {
    let hub = ProjectHub::in_memory();
    hub.requirements()
        .create(RequirementCreateInput {
            title: "Requirement".into(),
            ..Default::default()
        })
        .await
        .unwrap();

    let task = hub
        .create_task_linked(TaskCreateInput {
            title: "Implement REQ-001".into(),
            linked_requirements: vec!["REQ-001".into()],
            ..Default::default()
        })
        .await
        .unwrap();

    assert!(task.linked_requirements.contains(&"REQ-001".into()));

    let req = hub.requirements().get("REQ-001").await.unwrap();
    assert!(req.linked_task_ids.contains(&task.id));
}

#[tokio::test]
async fn delete_requirement() {
    let hub = ProjectHub::in_memory();
    hub.requirements()
        .create(RequirementCreateInput {
            title: "Doomed".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    hub.requirements().delete("REQ-001").await.unwrap();
    assert!(hub.requirements().get("REQ-001").await.is_err());
}
