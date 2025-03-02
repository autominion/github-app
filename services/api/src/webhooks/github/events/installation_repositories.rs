use async_trait::async_trait;
use database::{Database, NewRepository};
use serde::Deserialize;

use config::Config;
use github::{
    types::{Installation, Repo},
    GitHub,
};

use super::Event;

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum InstallationRepositoriesEvent {
    #[serde(rename = "added")]
    Added(Data),
    #[serde(rename = "removed")]
    Removed(Data),
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    repositories_added: Vec<Repo>,
    repositories_removed: Vec<Repo>,
    installation: Installation,
}

#[async_trait]
impl Event for InstallationRepositoriesEvent {
    async fn handle(self, _config: &Config, db: Database, _github: GitHub) {
        use InstallationRepositoriesEvent::*;
        println!("Installation repositories event received: {:?}", self);
        let mut conn = db.conn().await;
        let Data { repositories_added, repositories_removed, installation } = match self {
            Added(data) | Removed(data) => data,
            Other => return,
        };
        let installation = conn.get_installation_by_github_id(installation.id).await.unwrap();
        conn.update_or_add_installation_repositories(
            installation.id,
            repositories_added
                .into_iter()
                .map(|Repo { full_name, node_id, private, .. }| NewRepository {
                    github_full_name: full_name,
                    github_id: node_id,
                    github_private: private,
                })
                .collect(),
        )
        .await;
        conn.delete_installation_repositories_by_github_ids(
            installation.id,
            repositories_removed
                .into_iter()
                .map(|Repo { node_id, .. }| node_id)
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .await;
    }
}
