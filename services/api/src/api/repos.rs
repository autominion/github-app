use actix_web::{delete, get, post, put, web, HttpResponse};

use auth::UserSessionId;
use database::{Database, Update};
use github::GitHub;
use user_api::{AddRepoUserRequest, Repo, RepoUserInfo};
use uuid::Uuid;

#[get("/repos")]
pub async fn list_repos(user: UserSessionId, db: web::Data<Database>) -> HttpResponse {
    if !auth::user_is_active(&db, user.user_id).await {
        return HttpResponse::Forbidden().finish();
    }

    let mut conn = db.conn().await;
    let user_id = user.user_id;

    let repos = conn.repositories_for_user_access(user_id).await;

    let response = repos
        .into_iter()
        .map(|(repo, repo_inst, role)| Repo {
            id: repo.id.to_string(),
            name: repo.github_full_name,
            active: repo_inst.active,
            role: role.into(),
        })
        .collect::<Vec<_>>();

    HttpResponse::Ok().json(response)
}

#[get("/repos/{id}")]
pub async fn get_repo(
    user: UserSessionId,
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let user_id = user.user_id;
    let repo_id: Uuid = *path;

    if !auth::user_can_admin_repo(&db, user_id, repo_id).await {
        return HttpResponse::Forbidden().finish();
    }

    let mut conn = db.conn().await;

    let repo = conn.get_repository(&repo_id).await;
    let inst_repo = conn.installation_repository_by_repo_id(repo_id).await;

    let response = Repo {
        id: repo.id.to_string(),
        name: repo.github_full_name,
        active: inst_repo.active,
        role: user_api::UserRole::Admin,
    };

    HttpResponse::Ok().json(response)
}

#[delete("/repos/{id}/active")]
pub async fn repos_deactivate(
    user: UserSessionId,
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let user_id = user.user_id;
    let repo_id: Uuid = *path;

    if !auth::user_can_admin_repo(&db, user_id, repo_id).await {
        return HttpResponse::Forbidden().finish();
    }

    let mut conn = db.conn().await;

    let inst_repo = conn.installation_repository_by_repo_id(repo_id).await;

    let update = inst_repo.update().active(false);

    conn.update_installation_repository(update).await;

    HttpResponse::Ok().finish()
}

#[put("/repos/{id}/active")]
pub async fn repos_activate(
    user: UserSessionId,
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let user_id = user.user_id;
    let repo_id: Uuid = *path;

    if !auth::user_can_admin_repo(&db, user_id, repo_id).await {
        return HttpResponse::Forbidden().finish();
    }

    let mut conn = db.conn().await;

    let inst_repo = conn.installation_repository_by_repo_id(repo_id).await;

    let update = inst_repo.update().active(true);

    conn.update_installation_repository(update).await;

    HttpResponse::Ok().finish()
}

#[get("/repos/{id}/users")]
pub async fn list_repo_users(
    user: UserSessionId,
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let user_id = user.user_id;
    let repo_id: Uuid = *path;

    if !auth::user_can_admin_repo(&db, user_id, repo_id).await {
        return HttpResponse::Forbidden().finish();
    }

    let mut conn = db.conn().await;

    let inst_repo = conn.installation_repository_by_repo_id(repo_id).await;

    let users = conn
        .installation_repository_users(inst_repo.installation_id, inst_repo.repository_id)
        .await;

    let users = users
        .into_iter()
        .map(|user| RepoUserInfo {
            id: user.id.to_string(),
            name: user.github_name.unwrap_or_else(|| user.github_login.clone()),
            github_login: user.github_login,
        })
        .collect::<Vec<_>>();

    HttpResponse::Ok().json(users)
}

#[post("/repos/{id}/users")]
pub async fn add_repo_user(
    user: UserSessionId,
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    payload: web::Json<AddRepoUserRequest>,
    github: web::Data<GitHub>,
) -> HttpResponse {
    let user_id = user.user_id;
    let repo_id = path.into_inner();

    if !auth::user_can_admin_repo(&db, user_id, repo_id).await {
        return HttpResponse::Forbidden().finish();
    }

    let mut conn = db.conn().await;

    let inst_repo = conn.installation_repository_by_repo_id(repo_id).await;

    let jwt = github.github_app_jwt();
    let access_token = github.installation_access_token(&jwt).await;
    let github = github.with_access(&access_token.token);

    let Some(user_info) = github.user_info(&payload.github_login).await else {
        return HttpResponse::NotFound().finish();
    };

    let Some(existing_user) = conn.get_user_by_github_id(&user_info.id).await else {
        return HttpResponse::NotFound().finish();
    };

    conn.add_installation_repository_user(inst_repo.installation_id, repo_id, existing_user.id)
        .await;

    HttpResponse::Ok().finish()
}

#[delete("/repos/{id}/users/{user_id}")]
pub async fn delete_repo_user(
    user: UserSessionId,
    db: web::Data<Database>,
    path: web::Path<(Uuid, Uuid)>,
) -> HttpResponse {
    let user_id = user.user_id;
    let (repo_id, target_user_id) = path.into_inner();

    if !auth::user_can_admin_repo(&db, user_id, repo_id).await {
        return HttpResponse::Forbidden().finish();
    }

    let mut conn = db.conn().await;

    let inst_repo = conn.installation_repository_by_repo_id(repo_id).await;

    conn.delete_installation_repository_user(inst_repo.installation_id, repo_id, target_user_id)
        .await;

    HttpResponse::Ok().finish()
}
