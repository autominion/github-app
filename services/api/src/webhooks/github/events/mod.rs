use actix_web::web::Bytes;
use async_trait::async_trait;
use serde::Deserialize;

use config::Config;
use database::Database;
use github::GitHub;

mod installation;
mod installation_repositories;
mod issue_comment;
mod issues;
mod ping;

pub use installation::*;
pub use installation_repositories::*;
pub use issue_comment::*;
pub use issues::*;
pub use ping::*;

#[async_trait]
pub trait Event: for<'de> Deserialize<'de> {
    async fn handle(self, config: &Config, db: Database, github: GitHub);
    async fn handle_bytes(
        bytes: &Bytes,
        config: &Config,
        db: Database,
        github: GitHub,
    ) -> serde_json::Result<()> {
        let body = Self::parse(bytes)?;
        body.handle(config, db, github).await;
        Ok(())
    }
    fn parse(bytes: &Bytes) -> serde_json::Result<Self> {
        serde_json::from_slice(bytes)
    }
}
