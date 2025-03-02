use chrono::{DateTime, Utc};
use diesel::AsChangeset;
use diesel::Identifiable;
use diesel::{Insertable, Queryable};
use uuid::Uuid;

use crate::schema::tasks;
use crate::types::{TaskFailureReason, TaskStatus};
use crate::Update;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(belongs_to(Installation))]
#[diesel(belongs_to(Repository))]
#[diesel(table_name = tasks)]
pub struct Task {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by_id: Uuid,
    pub installation_id: Option<Uuid>,
    pub repository_id: Uuid,
    pub github_issue_id: String,
    pub github_issue_number: i64,
    pub status: TaskStatus,
    pub completion_description: Option<String>,
    pub failure_description: Option<String>,
    pub failure_reason: Option<TaskFailureReason>,
    pub agent_config_id: Option<Uuid>,
}

impl Update for Task {
    type Output = UpdateTask;

    fn update(&self) -> Self::Output {
        UpdateTask { id: self.id, ..Default::default() }
    }
}

#[derive(Default, AsChangeset, Identifiable)]
#[diesel(table_name = tasks)]
pub struct UpdateTask {
    pub id: Uuid,
    pub status: Option<TaskStatus>,
    pub agent_config_id: Option<Option<Uuid>>,
}

impl UpdateTask {
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn status(mut self, status: TaskStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn agent_config_id(mut self, agent_config_id: Option<Uuid>) -> Self {
        self.agent_config_id = Some(agent_config_id);
        self
    }
}

#[derive(Insertable)]
#[diesel(table_name = tasks)]
pub struct NewTask {
    pub installation_id: Uuid,
    pub repository_id: Uuid,
    pub created_by_id: Uuid,
    pub github_issue_id: String,
    pub github_issue_number: i64,
    pub status: TaskStatus,
    pub agent_config_id: Option<Uuid>,
}
