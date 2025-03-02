mod agent_configs;
mod conn;
mod installation_repository_users;
mod installation_users;
mod installations;
mod installations_repositories;
mod llm_interactions;
mod models;
mod repositories;
mod schema;
mod task_compute_usage;
mod tasks;
mod types;
mod users;

pub use conn::*;
pub use models::installations::*;
pub use models::installations_repositories::*;
pub use models::llm_interactions::*;
pub use models::repositories::*;
pub use models::tasks::*;
pub use models::users::*;
pub use types::*;

pub trait Update {
    type Output;

    fn update(&self) -> Self::Output;
}
