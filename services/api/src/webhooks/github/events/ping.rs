use async_trait::async_trait;
use serde::Deserialize;

use config::Config;
use database::Database;
use github::GitHub;

use super::Event;

#[derive(Deserialize)]
pub struct PingEvent {}

#[async_trait]
impl Event for PingEvent {
    async fn handle(self, _config: &Config, _db: Database, _github: GitHub) {}
}
