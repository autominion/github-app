use std::sync::Arc;
use std::time::Duration;

use auth::TokenSigner;
use config::Config;
use database::{Database, Task};
use github::GitHub;
use object_storage::S3;

mod git;
mod job;

#[derive(clap::Args, Clone)]
pub struct Args {
    #[clap(long, num_args = 0)]
    local: bool,
}

pub async fn exec(args: Args) {
    let config = Config::load();
    let github = GitHub::new(config.clone());
    let s3 = S3::new(&config).unwrap();
    let db = Database::connect(config.postgres_url.as_str()).await;
    let token_signer = Arc::new(auth::token_signer(&config));
    let mut conn = db.conn().await;
    loop {
        if let Some(task) = conn.receive_task().await {
            println!("Job received");
            tokio::spawn(handle_msg(
                config.clone(),
                args.clone(),
                db.clone(),
                github.clone(),
                s3.clone(),
                token_signer.clone(),
                task,
            ));
        } else {
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

async fn handle_msg(
    config: Config,
    args: Args,
    db: Database,
    github: GitHub,
    s3: S3,
    token_signer: Arc<TokenSigner>,
    task: Task,
) {
    let repo = db.conn().await.get_repository(&task.repository_id).await;

    let job = job::Job {
        issue_id: task.github_issue_id,
        repo_github_id: repo.github_id,
        repo_name: repo.github_full_name,
        task_id: task.id,
    };

    job::run(&config, &args, db.clone(), &github, &s3, &token_signer, &job).await;
}
