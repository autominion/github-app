use async_trait::async_trait;
use serde::Deserialize;

use config::Config;
use database::Database;
use github::{
    types::{Issue, Repo, User},
    GitHub,
};

use super::Event;

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
#[allow(dead_code)]
pub enum IssuesEvent {
    #[serde(rename = "opened")]
    Opened { repository: Repo, issue: Box<Issue>, sender: User },
    #[serde(other)]
    Other,
}

#[async_trait]
impl Event for IssuesEvent {
    async fn handle(self, _config: &Config, _db: Database, _github: GitHub) {}
}
