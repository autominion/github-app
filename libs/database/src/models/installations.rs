use chrono::{DateTime, Utc};
use diesel::{AsChangeset, Identifiable, Insertable, Queryable};
use uuid::Uuid;

use crate::schema::installations;
use crate::Update;

#[derive(Queryable, Identifiable)]
#[diesel(table_name = installations)]
pub struct Installation {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub github_id: i64,
    pub created_by_github_id: Option<String>,
    pub suspended_at: Option<DateTime<Utc>>,
    pub suspended_by_github_id: Option<String>,
}

impl Update for Installation {
    type Output = UpdateInstallationById;

    fn update(&self) -> Self::Output {
        UpdateInstallationById { id: self.id, ..Default::default() }
    }
}

#[derive(Default, AsChangeset, Identifiable)]
#[diesel(table_name = installations)]
pub struct UpdateInstallationById {
    id: Uuid,
    created_by_github_id: Option<Option<String>>,
    suspended_at: Option<Option<DateTime<Utc>>>,
    suspended_by_github_id: Option<Option<String>>,
}

impl UpdateInstallationById {
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn suspended_at(mut self, suspended_at: Option<DateTime<Utc>>) -> Self {
        self.suspended_at = Some(suspended_at);
        self
    }

    pub fn suspended_by_github_id(mut self, suspended_by_github_id: Option<String>) -> Self {
        self.suspended_by_github_id = Some(suspended_by_github_id);
        self
    }
}

#[derive(Default, AsChangeset)]
#[diesel(table_name = installations)]
pub struct UpdateInstallationByGitHubId {
    pub github_id: i64,
    created_by_github_id: Option<Option<String>>,
    suspended_at: Option<Option<DateTime<Utc>>>,
    suspended_by_github_id: Option<Option<String>>,
}

impl UpdateInstallationByGitHubId {
    pub fn new(github_id: i64) -> Self {
        Self { github_id, ..Default::default() }
    }

    pub fn suspended_at(mut self, suspended_at: Option<DateTime<Utc>>) -> Self {
        self.suspended_at = Some(suspended_at);
        self
    }

    pub fn suspended_by_github_id(mut self, suspended_by_github_id: Option<String>) -> Self {
        self.suspended_by_github_id = Some(suspended_by_github_id);
        self
    }
}

#[derive(AsChangeset, Insertable)]
#[diesel(table_name = installations)]
pub struct NewInstallation {
    pub github_id: i64,
    pub created_by_github_id: Option<Option<String>>,
}
