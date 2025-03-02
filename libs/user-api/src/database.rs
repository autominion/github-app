use crate::api::*;

impl From<database::TaskStatus> for TaskStatus {
    fn from(value: database::TaskStatus) -> Self {
        match value {
            database::TaskStatus::Queued => TaskStatus::Queued,
            database::TaskStatus::Running => TaskStatus::Running,
            database::TaskStatus::Completed => TaskStatus::Completed,
            database::TaskStatus::Failed => TaskStatus::Failed,
        }
    }
}

impl From<(database::Task, database::Repository)> for TaskInfo {
    fn from((task, repository): (database::Task, database::Repository)) -> Self {
        TaskInfo {
            id: task.id.to_string(),
            repo_name: repository.github_full_name,
            issue_number: task.github_issue_number,
            status: task.status.into(),
        }
    }
}

impl From<database::LLMInteraction> for LLMInteraction {
    fn from(value: database::LLMInteraction) -> Self {
        LLMInteraction {
            id: value.id.to_string(),
            request: value.request,
            response: value.response,
        }
    }
}

impl From<database::UserRole> for UserRole {
    fn from(value: database::UserRole) -> Self {
        match value {
            database::UserRole::Admin => UserRole::Admin,
            database::UserRole::Member => UserRole::Member,
        }
    }
}
