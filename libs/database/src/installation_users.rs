use diesel::expression::SelectableHelper;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::conn::Conn;
use crate::models::installation_users::NewInstallationUser;
use crate::schema::installation_users::dsl;
use crate::schema::users;
use crate::{User, UserRole};

impl Conn<'_> {
    pub async fn add_installation_user(&mut self, inst_id: Uuid, user_id: Uuid, role: UserRole) {
        let new_record = NewInstallationUser { installation_id: inst_id, user_id, role };

        diesel::insert_into(dsl::installation_users)
            .values(new_record)
            .on_conflict_do_nothing()
            .execute(&mut self.conn)
            .await
            .expect("Error inserting installation_repository_users");
    }

    /// A list of all users assocated with the installation.
    pub async fn installation_users_with_role(&mut self, inst_id: Uuid) -> Vec<(User, UserRole)> {
        dsl::installation_users
            .filter(dsl::installation_id.eq(inst_id))
            .inner_join(users::table)
            .select((User::as_select(), dsl::role))
            .load::<(User, UserRole)>(&mut self.conn)
            .await
            .expect("Error loading users for installation")
    }
}
