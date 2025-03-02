use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::conn::Conn;
use crate::models::users::{NewUser, UpdateUser, User};
use crate::schema::users;

impl Conn<'_> {
    pub async fn update_or_add_user(&mut self, new_user: NewUser) -> User {
        diesel::insert_into(users::table)
            .values(&new_user)
            .on_conflict(users::github_id)
            .do_update()
            .set(&new_user)
            .get_result(&mut self.conn)
            .await
            .unwrap()
    }

    pub async fn get_user(&mut self, user_id: &Uuid) -> User {
        users::table.find(user_id).first(&mut self.conn).await.unwrap()
    }

    pub async fn get_user_by_github_id(&mut self, github_id: &str) -> Option<User> {
        use crate::schema::users;

        users::table.filter(users::github_id.eq(github_id)).first(&mut self.conn).await.ok()
    }

    pub async fn update_user(&mut self, updated_user: UpdateUser) -> User {
        diesel::update(&updated_user).set(&updated_user).get_result(&mut self.conn).await.unwrap()
    }
}
