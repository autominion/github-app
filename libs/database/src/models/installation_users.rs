use chrono::{DateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::{schema::installation_users, UserRole};

#[derive(Queryable, Identifiable, Selectable)]
#[diesel(table_name = installation_users)]
#[diesel(primary_key(installation_id, user_id))]
pub struct InstallationUser {
    pub installation_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub role: UserRole,
}

#[derive(Insertable)]
#[diesel(table_name = installation_users)]
pub struct NewInstallationUser {
    pub installation_id: Uuid,
    pub user_id: Uuid,
    pub role: UserRole,
}
