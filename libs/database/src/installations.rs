use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::conn::Conn;
use crate::models::installations::{Installation, NewInstallation, UpdateInstallationByGitHubId};
use crate::schema::installations::dsl::*;

impl Conn<'_> {
    pub async fn update_or_add_installation(
        &mut self,
        new_installation: NewInstallation,
    ) -> Installation {
        diesel::insert_into(installations)
            .values(&new_installation)
            .on_conflict(github_id)
            .do_update()
            .set(&new_installation)
            .get_result(&mut self.conn)
            .await
            .unwrap()
    }

    pub async fn get_installation_by_github_id(&mut self, gh_id: i64) -> Option<Installation> {
        installations.filter(github_id.eq(gh_id)).first(&mut self.conn).await.ok()
    }

    pub async fn get_installation(&mut self, installation_id: &Uuid) -> Installation {
        installations.find(installation_id).first(&mut self.conn).await.unwrap()
    }

    pub async fn get_installations(&mut self) -> Vec<Installation> {
        installations.load(&mut self.conn).await.unwrap()
    }

    pub async fn update_installation_by_github_id(
        &mut self,
        update: UpdateInstallationByGitHubId,
    ) -> Installation {
        diesel::update(installations)
            .filter(github_id.eq(update.github_id))
            .set(update)
            .get_result(&mut self.conn)
            .await
            .unwrap()
    }

    pub async fn delete_installation_by_github_id(&mut self, gh_id: i64) {
        diesel::delete(installations.filter(github_id.eq(gh_id)))
            .execute(&mut self.conn)
            .await
            .unwrap();
    }
}
