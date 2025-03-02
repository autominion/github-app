use diesel::expression::SelectableHelper;
use diesel::{BoolExpressionMethods, CombineDsl, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::conn::Conn;
use crate::models::installation_repository_users::NewInstallationRepositoryUser;
use crate::schema::{
    installation_repository_users as iru, installation_users as iu,
    installations_repositories as ir, repositories, users,
};
use crate::{InstallationRepository, User, UserRole};

impl Conn<'_> {
    /// Inserts a record into `installation_repository_users`.
    pub async fn add_installation_repository_user(
        &mut self,
        inst_id: Uuid,
        repo_id: Uuid,
        user_id: Uuid,
    ) {
        let new_record = NewInstallationRepositoryUser {
            installation_id: inst_id,
            repository_id: repo_id,
            user_id,
        };

        diesel::insert_into(iru::table)
            .values(new_record)
            .on_conflict_do_nothing()
            .execute(&mut self.conn)
            .await
            .expect("Error inserting into installation_repository_users");
    }

    /// Returns all users who are explicitly in `installation_repository_users`
    /// for the given `(installation_id, repository_id)`.
    pub async fn installation_repository_users(
        &mut self,
        inst_id: Uuid,
        repo_id: Uuid,
    ) -> Vec<User> {
        iru::table
            .filter(iru::installation_id.eq(inst_id).and(iru::repository_id.eq(repo_id)))
            .inner_join(users::table)
            .select(User::as_select())
            .load(&mut self.conn)
            .await
            .expect("Error loading users for installation repository")
    }

    /// Deletes a single record from `installation_repository_users`.
    pub async fn delete_installation_repository_user(
        &mut self,
        inst_id: Uuid,
        repo_id: Uuid,
        user_id: Uuid,
    ) {
        diesel::delete(
            iru::table.filter(
                iru::installation_id
                    .eq(inst_id)
                    .and(iru::repository_id.eq(repo_id))
                    .and(iru::user_id.eq(user_id)),
            ),
        )
        .execute(&mut self.conn)
        .await
        .expect("Error deleting installation_repository_user");
    }

    /// Retrieves all users who are authorized for a given `(installation_id, repository_id)`.
    ///
    /// These are:
    /// - All users in `installation_users` for `installation_id`
    /// - All users in `installation_repository_users` for `(installation_id, repository_id)`
    ///
    /// We combine the two via `UNION` (which is inherently distinct).
    pub async fn authorized_users_for_repository(
        &mut self,
        inst_id: Uuid,
        repo_id: Uuid,
    ) -> Vec<User> {
        // Query #1: Users in `installation_repository_users`
        let repo_users_query = iru::table
            .filter(iru::installation_id.eq(&inst_id).and(iru::repository_id.eq(repo_id)))
            .inner_join(users::table)
            .select(User::as_select());

        // Query #2: Users in `installation_users`
        let inst_users_query = iu::table
            .filter(iu::installation_id.eq(&inst_id))
            .inner_join(users::table)
            .select(User::as_select());

        // Combine both queries using UNION (already distinct by default)
        let combined_query = repo_users_query.union(inst_users_query);

        combined_query.load(&mut self.conn).await.expect("Error loading authorized users")
    }

    /// Retrieves the `InstallationRepository` for a given `(github_user_id, github_repo_id)`
    /// if the user is authorized.
    ///
    /// The user is authorized if they appear in:
    ///   - `installation_users` for the same installation
    ///   - `installation_repository_users` for the same `(installation, repo)`
    ///
    /// We implement that via two queries that we `UNION`.
    pub async fn installation_repository_for_authorized_github_user(
        &mut self,
        github_user_id: &str,
        github_repo_id: &str,
    ) -> Option<InstallationRepository> {
        use crate::schema::repositories::dsl as repo_dsl;
        use crate::schema::users::dsl as user_dsl;

        // Query #1: user is in `installation_users` for the same installation
        let from_inst_users = ir::table
            .inner_join(repositories::table)
            .inner_join(iu::table.on(iu::installation_id.eq(ir::installation_id)))
            .inner_join(users::table.on(user_dsl::id.eq(iu::user_id)))
            .filter(
                repo_dsl::github_id.eq(github_repo_id).and(user_dsl::github_id.eq(github_user_id)),
            )
            .select(InstallationRepository::as_select());

        // Query #2: user is in `installation_repository_users` for the same `(installation, repo)`
        let from_inst_repo_users = ir::table
            .inner_join(repositories::table)
            .inner_join(
                iru::table.on(iru::installation_id
                    .eq(ir::installation_id)
                    .and(iru::repository_id.eq(ir::repository_id))),
            )
            .inner_join(users::table.on(user_dsl::id.eq(iru::user_id)))
            .filter(
                repo_dsl::github_id.eq(github_repo_id).and(user_dsl::github_id.eq(github_user_id)),
            )
            .select(InstallationRepository::as_select());

        // Combine both queries with `UNION`, which is inherently distinct in SQL
        let combined_query = from_inst_users.union(from_inst_repo_users);

        // Because `.first()` does not work on a union, load all and pick the first
        if let Ok(results) = combined_query.load::<InstallationRepository>(&mut self.conn).await {
            results.into_iter().next()
        } else {
            None
        }
    }

    pub async fn user_is_member_of_repository(&mut self, user_id: Uuid, repo_id: Uuid) -> bool {
        // Query #1: user is in `installation_users` for the same installation
        let from_inst_users = ir::table
            .inner_join(iu::table.on(iu::installation_id.eq(ir::installation_id)))
            .filter(ir::repository_id.eq(repo_id).and(iu::user_id.eq(user_id)))
            .select(InstallationRepository::as_select());

        // Query #2: user is in `installation_repository_users` for the same (installation, repo)
        let from_inst_repo_users = ir::table
            .inner_join(
                iru::table.on(iru::installation_id
                    .eq(ir::installation_id)
                    .and(iru::repository_id.eq(ir::repository_id))),
            )
            .filter(ir::repository_id.eq(repo_id).and(iru::user_id.eq(user_id)))
            .select(InstallationRepository::as_select());

        let combined_query = from_inst_users.union(from_inst_repo_users);

        // Load the results. If there's at least one row, the user has access.
        match combined_query.load::<InstallationRepository>(&mut self.conn).await {
            Ok(results) => !results.is_empty(),
            Err(_) => false,
        }
    }

    /// Checks if the user is an admin of the repository.
    pub async fn user_is_admin_of_repository(&mut self, user_id: Uuid, repo_id: Uuid) -> bool {
        let admin_query = ir::table
            .inner_join(
                iu::table
                    .on(iu::installation_id.eq(ir::installation_id).and(iu::user_id.eq(user_id))),
            )
            .filter(ir::repository_id.eq(repo_id))
            .filter(iu::role.eq(UserRole::Admin))
            .select(InstallationRepository::as_select());

        match admin_query.load::<InstallationRepository>(&mut self.conn).await {
            Ok(results) => !results.is_empty(),
            Err(_) => false,
        }
    }
}
