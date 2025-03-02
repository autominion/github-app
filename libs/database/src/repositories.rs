use diesel::upsert::excluded;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::conn::Conn;
use crate::models::repositories::{NewRepository, Repository, UpdateRepository};
use crate::schema::repositories::dsl::*;

impl Conn<'_> {
    pub async fn update_or_add_repositories(
        &mut self,
        new_repositories: Vec<NewRepository>,
    ) -> Vec<Repository> {
        diesel::insert_into(repositories)
            .values(new_repositories)
            .on_conflict(github_id)
            .do_update()
            .set((
                github_full_name.eq(excluded(github_full_name)),
                github_private.eq(excluded(github_private)),
            ))
            .get_results(&mut self.conn)
            .await
            .unwrap()
    }

    pub async fn get_repository(&mut self, repository_id: &Uuid) -> Repository {
        repositories.find(repository_id).first(&mut self.conn).await.unwrap()
    }

    pub async fn get_repository_by_full_name(&mut self, full_name: &str) -> Repository {
        repositories.filter(github_full_name.eq(full_name)).first(&mut self.conn).await.unwrap()
    }

    pub async fn update_repository(&mut self, updated_repository: UpdateRepository) -> Repository {
        diesel::update(repositories)
            .set(updated_repository)
            .get_result(&mut self.conn)
            .await
            .unwrap()
    }
}
