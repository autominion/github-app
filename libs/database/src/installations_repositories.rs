use diesel::expression::SelectableHelper;
use diesel::{BoolExpressionMethods, CombineDsl, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::conn::Conn;
use crate::models::installations_repositories::NewInstallationRepository;
use crate::models::repositories::{NewRepository, Repository};
use crate::schema::{
    installation_repository_users as iru, installation_users as iu,
    installations_repositories as ir, repositories,
};
use crate::{InstallationRepository, UpdateInstallationRepository, UserRole};

impl Conn<'_> {
    pub async fn update_or_add_installation_repositories(
        &mut self,
        inst_id: Uuid,
        new_repositories: Vec<NewRepository>,
    ) -> Vec<Repository> {
        let repos = self.update_or_add_repositories(new_repositories).await;

        let new_installation_repositories: Vec<NewInstallationRepository> = repos
            .iter()
            .map(|repo| NewInstallationRepository {
                installation_id: inst_id,
                repository_id: repo.id,
            })
            .collect();

        diesel::insert_into(ir::table)
            .values(new_installation_repositories)
            .on_conflict_do_nothing()
            .execute(&mut self.conn)
            .await
            .unwrap();

        repos
    }

    pub async fn update_installation_repository(
        &mut self,
        update: UpdateInstallationRepository,
    ) -> InstallationRepository {
        diesel::update(&update).set(&update).get_result(&mut self.conn).await.unwrap()
    }

    pub async fn delete_installation_repositories_by_github_ids(
        &mut self,
        inst_id: Uuid,
        repo_ids: &[String],
    ) {
        let repo_ids = repositories::dsl::repositories
            .filter(repositories::dsl::github_id.eq_any(repo_ids))
            .select(repositories::dsl::id)
            .load::<Uuid>(&mut self.conn)
            .await
            .unwrap();

        diesel::delete(
            ir::table
                .filter(ir::installation_id.eq(inst_id))
                .filter(ir::repository_id.eq_any(repo_ids)),
        )
        .execute(&mut self.conn)
        .await
        .unwrap();
    }

    pub async fn installation_repository_by_full_name(
        &mut self,
        inst_id: Uuid,
        repo_full_name: &str,
    ) -> Repository {
        ir::table
            .filter(ir::installation_id.eq(inst_id))
            .inner_join(repositories::dsl::repositories)
            .filter(repositories::dsl::github_full_name.eq(repo_full_name))
            .select(Repository::as_select())
            .first(&mut self.conn)
            .await
            .unwrap()
    }

    pub async fn installation_repositories(&mut self, inst_id: Uuid) -> Vec<Repository> {
        ir::table
            .filter(ir::installation_id.eq(inst_id))
            .inner_join(repositories::dsl::repositories)
            .select(Repository::as_select())
            .load(&mut self.conn)
            .await
            .unwrap()
    }

    pub async fn installation_repository_by_repo_id(
        &mut self,
        repo_id: Uuid,
    ) -> InstallationRepository {
        // Invariant: there should only be one installation repository for a given repo.
        ir::table
            .filter(ir::repository_id.eq(repo_id))
            .inner_join(repositories::dsl::repositories)
            .select(InstallationRepository::as_select())
            .first(&mut self.conn)
            .await
            .unwrap()
    }

    /// Returns all `(Repository, InstallationRepository)` pairs to which `user_id`
    /// has access. A user has access if:
    /// - They appear in `installation_users` for that installation (all repos),
    /// - OR they appear in `installation_repository_users` for that `(installation, repo)`.
    pub async fn repositories_for_user_access(
        &mut self,
        user_id: Uuid,
    ) -> Vec<(Repository, InstallationRepository, UserRole)> {
        use diesel::dsl::sql;

        // Query #1: All repos from installations where user is in `installation_users`.
        let from_inst_users = ir::table
            .inner_join(repositories::table)
            .inner_join(iu::table.on(iu::installation_id.eq(ir::installation_id)))
            .filter(iu::user_id.eq(user_id))
            .select((Repository::as_select(), InstallationRepository::as_select(), iu::role));

        // Query #2: All repos from installations where user is in `installation_repository_users`.
        let from_inst_repo_users = ir::table
            .inner_join(repositories::table)
            .inner_join(
                iru::table.on(iru::installation_id
                    .eq(ir::installation_id)
                    .and(iru::repository_id.eq(ir::repository_id))),
            )
            .filter(iru::user_id.eq(user_id))
            .select((
                Repository::as_select(),
                InstallationRepository::as_select(),
                sql::<crate::schema::sql_types::UserRole>("'member'"),
            ));

        let combined_query = from_inst_users.union(from_inst_repo_users);

        combined_query
            .load(&mut self.conn)
            .await
            .expect("Error loading repositories for user admin access")
    }
}
