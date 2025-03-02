use async_trait::async_trait;
use auth::CreateTask;
use serde::Deserialize;

use config::Config;
use database::{Database, NewTask, TaskStatus};
use github::types::{Comment, Issue, Repo, User};
use github::GitHub;

use super::Event;

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum IssueCommentEvent {
    #[serde(rename = "created")]
    Created { repository: Repo, issue: Box<Issue>, comment: Box<Comment>, sender: User },
    #[serde(other)]
    Other,
}

#[async_trait]
impl Event for IssueCommentEvent {
    async fn handle(self, config: &Config, db: Database, _github: GitHub) {
        let IssueCommentEvent::Created { repository, issue, comment, sender } = self else {
            return;
        };

        println!("{}", comment.body);

        if comment.body != format!("{} solve", config.github_bot_handle) {
            return;
        }

        println!("Checking if user is authorized");

        println!("{}", sender.node_id);
        println!("{}", repository.node_id);

        let Some(CreateTask { inst_repo, user }) =
            auth::github_user_can_create_task(&db, &sender.node_id, &repository.node_id).await
        else {
            return;
        };

        let new_task = NewTask {
            installation_id: inst_repo.installation_id,
            repository_id: inst_repo.repository_id,
            created_by_id: user.id,
            github_issue_id: issue.node_id,
            github_issue_number: issue.number,
            status: TaskStatus::Queued,
            agent_config_id: None,
        };

        println!("Adding task to queue");

        let mut conn = db.conn().await;
        conn.add_task(new_task).await;
    }
}
