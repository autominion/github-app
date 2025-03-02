//! The purpose of `sync` is to synchronize the local database with external sources.
//! In particular, it will synchronize all installation repositories from GitHub.
//! This is useful when the local database is out of date because of a reinstallation or an outage.

use uuid::Uuid;

use config::Config;
use database::{Database, NewInstallation, NewRepository, NewUser, UserRole};
use github::{AccountType, GitHub, User};

async fn synchronize_installations(config: Config, db: Database, github: GitHub) {
    let installation_id = config.github_app_installation_id;
    let app_jwt = github.github_app_jwt();
    let github_installations = github.installations(&app_jwt).await;
    let github_installation = github_installations
        .into_iter()
        .find(|installation| installation.id == installation_id)
        .expect("GitHub App installation not found");

    synchronize_installation(github_installation, db.clone(), github.clone()).await;
}

async fn synchronize_installation(
    github_installation: github::Installation,
    db: Database,
    github: GitHub,
) {
    let mut conn = db.conn().await;

    let installation = conn
        .update_or_add_installation(NewInstallation {
            github_id: github_installation.id,
            created_by_github_id: None,
        })
        .await;
    let members = if github_installation.account.r#type == AccountType::Organization {
        let org = github_installation.account.login;
        let app_jwt = github.github_app_jwt();
        let access_token = github.installation_access_token(&app_jwt).await;
        let github = github.with_access(&access_token.token);
        let members = github.organization_members(&org).await;
        let mut members_with_role = vec![];
        for member in members {
            let membership = github.organization_membership(&org, &member.login).await;
            let role = match membership.role {
                github::Role::Admin => UserRole::Admin,
                github::Role::Member => UserRole::Member,
                github::Role::Unknown => UserRole::Member,
            };
            members_with_role.push((member, role));
        }
        members_with_role
    } else {
        vec![(
            User {
                node_id: github_installation.account.node_id,
                login: github_installation.account.login,
                name: None,
            },
            UserRole::Admin,
        )]
    };
    for (member, role) in members {
        let new_user = NewUser {
            github_id: member.node_id,
            github_email: None,
            github_name: None,
            github_login: member.login,
            github_access_token: None,
            github_access_token_expires_at: None,
        };
        let user = conn.update_or_add_user(new_user).await;
        conn.add_installation_user(installation.id, user.id, role).await;
    }

    drop(conn);

    synchronize_installation_repositories(installation.id, db, github).await;
}

async fn synchronize_installation_repositories(
    installation_id: Uuid,
    db: Database,
    github: GitHub,
) {
    let jwt = github.github_app_jwt();
    let access_token = github.installation_access_token(&jwt).await;
    let github = github.with_access(&access_token.token);
    let repos = github.installation_repositories().await;
    let mut conn = db.conn().await;
    let new_repositories = repos
        .into_iter()
        .map(|repo| NewRepository {
            github_full_name: repo.full_name,
            github_id: repo.node_id,
            github_private: repo.private,
        })
        .collect();
    conn.update_or_add_installation_repositories(installation_id, new_repositories).await;
}

#[tokio::main]
async fn main() {
    let config = Config::load();
    let db = Database::connect_and_init(config.postgres_url.as_str()).await;
    let github = GitHub::new(config.clone());

    synchronize_installations(config, db, github).await;
}
