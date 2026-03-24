use ao_projects_protocol::*;

#[test]
fn parse_task_status() {
    assert_eq!("ready".parse::<TaskStatus>().unwrap(), TaskStatus::Ready);
    assert_eq!("in_progress".parse::<TaskStatus>().unwrap(), TaskStatus::InProgress);
    assert_eq!("in-progress".parse::<TaskStatus>().unwrap(), TaskStatus::InProgress);
    assert_eq!("done".parse::<TaskStatus>().unwrap(), TaskStatus::Done);
    assert_eq!("BLOCKED".parse::<TaskStatus>().unwrap(), TaskStatus::Blocked);
    assert!("invalid".parse::<TaskStatus>().is_err());
}

#[test]
fn parse_priority() {
    assert_eq!("critical".parse::<Priority>().unwrap(), Priority::Critical);
    assert_eq!("HIGH".parse::<Priority>().unwrap(), Priority::High);
    assert_eq!("medium".parse::<Priority>().unwrap(), Priority::Medium);
    assert_eq!("low".parse::<Priority>().unwrap(), Priority::Low);
}

#[test]
fn parse_task_type() {
    assert_eq!("feature".parse::<TaskType>().unwrap(), TaskType::Feature);
    assert_eq!("bugfix".parse::<TaskType>().unwrap(), TaskType::Bugfix);
    assert_eq!("docs".parse::<TaskType>().unwrap(), TaskType::Docs);
}

#[test]
fn parse_requirement_status() {
    assert_eq!("draft".parse::<RequirementStatus>().unwrap(), RequirementStatus::Draft);
    assert_eq!("refined".parse::<RequirementStatus>().unwrap(), RequirementStatus::Refined);
    assert_eq!("planned".parse::<RequirementStatus>().unwrap(), RequirementStatus::Planned);
    assert_eq!("po-review".parse::<RequirementStatus>().unwrap(), RequirementStatus::PoReview);
    assert_eq!("em-review".parse::<RequirementStatus>().unwrap(), RequirementStatus::EmReview);
    assert_eq!("needs-rework".parse::<RequirementStatus>().unwrap(), RequirementStatus::NeedsRework);
    assert_eq!("approved".parse::<RequirementStatus>().unwrap(), RequirementStatus::Approved);
    assert_eq!("implemented".parse::<RequirementStatus>().unwrap(), RequirementStatus::Implemented);
}

#[test]
fn parse_requirement_priority() {
    assert_eq!("must".parse::<RequirementPriority>().unwrap(), RequirementPriority::Must);
    assert_eq!("should".parse::<RequirementPriority>().unwrap(), RequirementPriority::Should);
    assert_eq!("could".parse::<RequirementPriority>().unwrap(), RequirementPriority::Could);
    assert_eq!("wont".parse::<RequirementPriority>().unwrap(), RequirementPriority::Wont);
}

#[test]
fn display_round_trip() {
    let status = TaskStatus::InProgress;
    let s = status.to_string();
    assert_eq!(s, "in-progress");
    assert_eq!(s.parse::<TaskStatus>().unwrap(), TaskStatus::InProgress);
    assert_eq!("in_progress".parse::<TaskStatus>().unwrap(), TaskStatus::InProgress);
}

#[test]
fn priority_to_task_priority() {
    assert_eq!(RequirementPriority::Must.to_task_priority(), Priority::High);
    assert_eq!(RequirementPriority::Should.to_task_priority(), Priority::Medium);
    assert_eq!(RequirementPriority::Could.to_task_priority(), Priority::Low);
}
