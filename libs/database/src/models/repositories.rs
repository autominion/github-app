use chrono::{DateTime, Utc};
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::schema::repositories;
use crate::Update;

#[derive(Queryable, Identifiable, Selectable)]
#[diesel(table_name = repositories)]
pub struct Repository {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub github_id: String,
    pub github_full_name: String,
    pub github_private: bool,
    pub default_agent_config_id: Option<Uuid>,
}

impl Update for Repository {
    type Output = UpdateRepository;

    fn update(&self) -> Self::Output {
        UpdateRepository { id: self.id, ..Default::default() }
    }
}

#[derive(Default, AsChangeset, Identifiable)]
#[diesel(table_name = repositories)]
pub struct UpdateRepository {
    id: Uuid,
    github_full_name: Option<String>,
    github_private: Option<bool>,
    default_agent_config_id: Option<Option<Uuid>>,
}

impl UpdateRepository {
    pub fn id(mut self, id: Uuid) -> UpdateRepository {
        self.id = id;
        self
    }

    pub fn github_full_name(mut self, github_full_name: String) -> UpdateRepository {
        self.github_full_name = Some(github_full_name);
        self
    }

    pub fn github_private(mut self, github_private: bool) -> UpdateRepository {
        self.github_private = Some(github_private);
        self
    }

    pub fn default_agent_config_id(mut self, default_agent_config_id: Option<Uuid>) -> Self {
        self.default_agent_config_id = Some(default_agent_config_id);
        self
    }
}

#[derive(Insertable)]
#[diesel(table_name = repositories)]
pub struct NewRepository {
    pub github_id: String,
    pub github_full_name: String,
    pub github_private: bool,
}

impl NewRepository {
    pub fn into_update(self, id: Uuid) -> UpdateRepository {
        let NewRepository { github_full_name, .. } = self;

        UpdateRepository::default().id(id).github_full_name(github_full_name)
    }
}
