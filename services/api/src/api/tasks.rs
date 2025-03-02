use actix_web::{get, web, HttpResponse};
use serde::Deserialize;

use database::Database;
use object_storage::{GetObjectError, S3};
use uuid::Uuid;

use auth::UserSessionId;
use user_api::{TaskDetails, TaskInfo, TaskPollResponse};

#[get("/tasks")]
pub async fn list_tasks(user: UserSessionId, db: web::Data<Database>) -> HttpResponse {
    if !auth::user_is_active(&db, user.user_id).await {
        return HttpResponse::Forbidden().finish();
    };

    let mut conn = db.conn().await;

    let tasks = conn.get_tasks_for_user(&user.user_id).await;

    let tasks: Vec<TaskInfo> = tasks.into_iter().map(Into::into).collect();

    HttpResponse::Ok().json(tasks)
}

#[get("/tasks/{id}")]
pub async fn task_details(
    user: UserSessionId,
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let task_id: Uuid = path.into_inner();

    if !auth::user_can_read_task(&db, user.user_id, task_id).await {
        return HttpResponse::Forbidden().finish();
    }

    let mut conn = db.conn().await;

    let (task, repo) = conn.get_task_and_repository(&task_id).await;
    let interactions = conn.llm_interactions(&task_id).await;

    let interactions = interactions.into_iter().map(Into::into).collect();

    let response = TaskDetails {
        id: task_id.to_string(),
        repo_name: repo.github_full_name,
        issue_number: task.github_issue_number,
        status: task.status.into(),
        interactions,
    };

    HttpResponse::Ok().json(response)
}

#[get("/tasks/{id}/logs")]
pub async fn task_logs(
    user: UserSessionId,
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    s3: web::Data<S3>,
) -> HttpResponse {
    let task_id = path.into_inner();

    if !auth::user_can_read_task(&db, user.user_id, task_id).await {
        return HttpResponse::Forbidden().finish();
    }

    match s3.log_for_task(&task_id).await {
        Ok(log) => HttpResponse::Ok().content_type("text/plain").body(log),
        Err(GetObjectError::NotFound) => HttpResponse::NotFound().finish(),
        Err(GetObjectError::Unexpected(err)) => {
            log::error!("Failed to get task log: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Deserialize)]
pub struct PollQuery {
    after: Uuid,
}

#[get("/tasks/{id}/poll")]
pub async fn task_poll(
    user: UserSessionId,
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    query: web::Query<PollQuery>,
) -> HttpResponse {
    let task_id: Uuid = path.into_inner();

    if !auth::user_can_read_task(&db, user.user_id, task_id).await {
        return HttpResponse::Forbidden().finish();
    }

    let mut conn = db.conn().await;

    let task_status = conn.get_task_status(&task_id).await;
    let interactions = conn.llm_interactions_after(task_id, query.after).await;
    let interactions = interactions.into_iter().map(Into::into).collect();

    let response = TaskPollResponse { status: task_status.into(), interactions };

    HttpResponse::Ok().json(response)
}
