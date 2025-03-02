use chrono::{DateTime, Utc};
use diesel::prelude::Associations;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::schema::installation_repository_users;
use crate::{Installation, Repository, User};

#[derive(Queryable, Identifiable, Selectable, Associations)]
#[diesel(
    belongs_to(Installation, foreign_key = installation_id)
)]
#[diesel(
    belongs_to(Repository, foreign_key = repository_id)
)]
#[diesel(belongs_to(User))]
#[diesel(table_name = installation_repository_users)]
#[diesel(primary_key(installation_id, repository_id, user_id))]
pub struct InstallationRepositoryUser {
    pub installation_id: Uuid,
    pub repository_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = installation_repository_users)]
pub struct NewInstallationRepositoryUser {
    pub installation_id: Uuid,
    pub repository_id: Uuid,
    pub user_id: Uuid,
}
