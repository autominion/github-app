use actix_web::{delete, get, put, web, HttpResponse};
use chrono::offset::Utc;

use auth::UserSessionId;
use database::{Database, UpdateUser};
use user_api::{OpenRouterStatus, UserInfo};

#[get("/user/info")]
async fn user_info(user: UserSessionId, db: web::Data<Database>) -> HttpResponse {
    let mut conn = db.conn().await;
    let user = conn.get_user(&user.user_id).await;

    // Invariant: user has a github_email
    // The GitHub email address is filled in when the user logs in with GitHub for the first time.
    let email_domain = user.github_email.unwrap().split('@').nth(1).unwrap().to_owned();

    let user_info = UserInfo {
        name: user.github_name.unwrap_or(user.github_login),
        email_domain,
        active: user.active,
        on_waitlist: user.joined_waitlist_at.is_some(),
    };

    HttpResponse::Ok().json(user_info)
}

#[put("/user/waitlist")]
async fn waitlist_join(user: UserSessionId, db: web::Data<Database>) -> HttpResponse {
    if auth::user_is_active(&db, user.user_id).await {
        return HttpResponse::Forbidden().finish();
    }

    let mut conn = db.conn().await;
    let now = Utc::now();
    let update_user = UpdateUser::default().id(user.user_id).joined_waitlist_at(Some(now));
    conn.update_user(update_user).await;

    HttpResponse::Ok().finish()
}

#[delete("/user/waitlist")]
async fn waitlist_leave(user: UserSessionId, db: web::Data<Database>) -> HttpResponse {
    if auth::user_is_active(&db, user.user_id).await {
        return HttpResponse::Forbidden().finish();
    }

    let mut conn = db.conn().await;
    let update_user = UpdateUser::default().id(user.user_id).joined_waitlist_at(None);
    conn.update_user(update_user).await;

    HttpResponse::Ok().finish()
}

#[get("/user/openrouter")]
async fn openrouter_status(session_id: UserSessionId, db: web::Data<Database>) -> HttpResponse {
    if !auth::user_is_active(&db, session_id.user_id).await {
        return HttpResponse::Forbidden().finish();
    }

    let mut conn = db.conn().await;
    let user = conn.get_user(&session_id.user_id).await;
    let status = OpenRouterStatus { connected: user.openrouter_key.is_some() };
    HttpResponse::Ok().json(status)
}

#[delete("/user/openrouter")]
async fn disconnect_openrouter(session_id: UserSessionId, db: web::Data<Database>) -> HttpResponse {
    if !auth::user_is_active(&db, session_id.user_id).await {
        return HttpResponse::Forbidden().finish();
    }

    let mut conn = db.conn().await;
    conn.update_user(UpdateUser::default().id(session_id.user_id).openrouter_key(None)).await;
    HttpResponse::Ok().finish()
}
