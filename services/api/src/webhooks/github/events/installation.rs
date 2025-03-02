use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;

use config::Config;
use database::{
    Database, NewInstallation, NewRepository, NewUser, UpdateInstallationByGitHubId, UserRole,
};
use github::{
    types::{Installation, Repo, User},
    AccountType, GitHub,
};

use super::Event;

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
#[allow(dead_code)]
pub enum InstallationEvent {
    #[serde(rename = "created")]
    Created { sender: User, repositories: Vec<Repo>, installation: Installation },
    #[serde(rename = "deleted")]
    Deleted { sender: User, installation: Installation },
    #[serde(rename = "new_permissions_accepted")]
    NewPermissionsAccepted { sender: User, installation: Installation },
    #[serde(rename = "suspend")]
    Suspend { sender: User, installation: Installation },
    #[serde(rename = "unsuspend")]
    Unsuspend { sender: User, installation: Installation },
    #[serde(other)]
    Other,
}

#[async_trait]
impl Event for InstallationEvent {
    async fn handle(self, _config: &Config, db: Database, github: GitHub) {
        use InstallationEvent::*;
        let mut conn = db.conn().await;
        println!("Installation event received: {:?}", self);
        match self {
            Created { sender, installation: github_installation, repositories } => {
                let installation = conn
                    .update_or_add_installation(NewInstallation {
                        github_id: github_installation.id,
                        created_by_github_id: Some(Some(sender.node_id)),
                    })
                    .await;
                // TODO: Consider running the following code detached from this event handler
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
                conn.update_or_add_installation_repositories(
                    installation.id,
                    repositories
                        .into_iter()
                        .map(|Repo { full_name, node_id, private, .. }| NewRepository {
                            github_full_name: full_name,
                            github_id: node_id,
                            github_private: private,
                        })
                        .collect(),
                )
                .await;
            }
            Deleted { installation, .. } => {
                conn.delete_installation_by_github_id(installation.id).await
            }
            Suspend { installation, sender } => {
                let update = UpdateInstallationByGitHubId::new(installation.id)
                    .suspended_at(Some(Utc::now()))
                    .suspended_by_github_id(Some(sender.node_id));
                conn.update_installation_by_github_id(update).await;
            }
            Unsuspend { installation, .. } => {
                let update = UpdateInstallationByGitHubId::new(installation.id)
                    .suspended_at(None)
                    .suspended_by_github_id(None);
                conn.update_installation_by_github_id(update).await;
            }
            NewPermissionsAccepted { .. } => (),
            Other => (),
        }
    }
}
