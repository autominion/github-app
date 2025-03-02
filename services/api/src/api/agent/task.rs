use actix_web::{get, post, web, HttpResponse};

use agent_api::types::task::*;
use auth::AgentSessionId;
use config::Config;
use database::Database;
use github::GitHub;

#[get("/task")]
pub async fn task_info(
    config: web::Data<Config>,
    agent: AgentSessionId,
    db: web::Data<Database>,
    github: web::Data<GitHub>,
) -> HttpResponse {
    let mut conn = db.conn().await;

    let task_id = agent.task_id;

    let (task, _repo) = conn.get_task_and_repository(&task_id).await;

    let jwt = github.github_app_jwt();
    let access_token = github.installation_access_token(&jwt).await;
    let github = github.with_access(&access_token.token);

    let issue_info = github.issue_info(&task.github_issue_id).await;

    let git_repo_url = config.web_base_url.join("/api/agent/git").unwrap();
    // For now we assume that the task id is the branch name
    let git_branch = task.id.to_string();

    let response = Task {
        status: task.status.into(),
        description: issue_info.body,
        git_user_name: config.github_git_name.clone(),
        git_user_email: config.github_git_email.clone(),
        git_repo_url,
        git_branch,
    };

    HttpResponse::Ok().json(response)
}

#[post("/task/complete")]
pub async fn task_complete(
    agent: AgentSessionId,
    db: web::Data<Database>,
    body: web::Json<TaskComplete>,
) -> HttpResponse {
    let mut conn = db.conn().await;

    if let Err(err) = except_task_running(&mut conn, &agent).await {
        return err;
    }

    let task_id = agent.task_id;

    conn.complete_task(&task_id, &body.description).await;

    HttpResponse::Ok().finish()
}

#[post("/task/fail")]
pub async fn task_fail(
    agent: AgentSessionId,
    db: web::Data<Database>,
    body: web::Json<TaskFailure>,
) -> HttpResponse {
    let mut conn = db.conn().await;

    if let Err(err) = except_task_running(&mut conn, &agent).await {
        return err;
    }

    let task_id = agent.task_id;

    conn.fail_task(&task_id, body.reason.map(Into::into), &body.description).await;

    HttpResponse::Ok().finish()
}

pub async fn except_task_running(
    conn: &mut database::Conn<'_>,
    agent: &AgentSessionId,
) -> Result<(), HttpResponse> {
    let task_status: TaskStatus = conn.get_task_status(&agent.task_id).await.into();
    if task_status == TaskStatus::Running {
        Ok(())
    } else {
        Err(HttpResponse::BadRequest().body("Task is not running"))
    }
}
