use chrono::{DateTime, Utc};
use diesel::prelude::Associations;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::schema::installations_repositories;
use crate::{Installation, Repository, Update};

#[derive(Queryable, Identifiable, Selectable, Associations)]
#[diesel(belongs_to(Installation))]
#[diesel(belongs_to(Repository))]
#[diesel(table_name = installations_repositories)]
#[diesel(primary_key(installation_id, repository_id))]
pub struct InstallationRepository {
    pub installation_id: Uuid,
    pub repository_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub active: bool,
}

impl Update for InstallationRepository {
    type Output = UpdateInstallationRepository;

    fn update(&self) -> Self::Output {
        UpdateInstallationRepository {
            installation_id: self.installation_id,
            repository_id: self.repository_id,
            ..Default::default()
        }
    }
}

#[derive(Default, AsChangeset, Identifiable)]
#[diesel(table_name = installations_repositories)]
#[diesel(primary_key(installation_id, repository_id))]
pub struct UpdateInstallationRepository {
    pub installation_id: Uuid,
    pub repository_id: Uuid,
    pub active: Option<bool>,
}

impl UpdateInstallationRepository {
    pub fn active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }
}

#[derive(Insertable)]
#[diesel(table_name = installations_repositories)]
pub struct NewInstallationRepository {
    pub installation_id: Uuid,
    pub repository_id: Uuid,
}
