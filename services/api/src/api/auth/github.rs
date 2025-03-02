use actix_jwt_auth_middleware::{AuthError, AuthResult};
use actix_web::ResponseError;
use actix_web::{get, http::header::LOCATION, web, HttpResponse};
use chrono::offset::Utc;
use chrono::Duration;
use serde::Deserialize;
use thiserror::Error;

use ::auth::{new_session_id, SessionId, TokenSigner, UserSessionId};
use config::{AccessControl, Config};
use database::Database;
use github::urls::OAUTH_AUTHORIZE_URL;
use github::GitHub;

#[get("/auth")]
async fn auth(config: web::Data<Config>) -> HttpResponse {
    let mut location = OAUTH_AUTHORIZE_URL.clone();
    let redirect_uri = config.web_base_url.join("/auth-code").unwrap();
    location
        .query_pairs_mut()
        .append_pair("client_id", &config.github_app_client_id)
        .append_pair("redirect_uri", redirect_uri.as_str());
    HttpResponse::TemporaryRedirect().append_header((LOCATION, location.as_str())).finish()
}

#[derive(Deserialize)]
struct LoginCode {
    code: String,
}

#[get("/auth-code")]
async fn auth_code(
    query: web::Query<LoginCode>,
    token_signer: web::Data<TokenSigner>,
    config: web::Data<Config>,
    github: web::Data<GitHub>,
    db: web::Data<Database>,
) -> AuthResult<HttpResponse> {
    let mut db_conn = db.conn().await;

    let now = Utc::now();
    let gh_access = github.get_ref().user_access_token(&query.code).await;
    let gh_token_expires_at = now + Duration::seconds(gh_access.expires_in);
    let github = github.get_ref().with_access(&gh_access.access_token);
    let gh_user = github.viewer_info().await;
    let gh_email = github.user_email().await;

    match config.access_control {
        AccessControl::Allowlist => {
            if !config.whitelisted_emails.contains(&gh_email) {
                return Err(AuthError::RefreshAuthorizerDenied(
                    AuthCodeError::RestrictedByAllowlist.into(),
                ));
            }
        }
        AccessControl::Waitlist => (),
    };

    let new_user = database::NewUser {
        github_id: gh_user.id,
        github_name: Some(gh_user.name),
        github_email: Some(Some(gh_email.clone())),
        github_login: gh_user.login,
        github_access_token: Some(Some(gh_access.access_token.clone())),
        github_access_token_expires_at: Some(Some(gh_token_expires_at)),
    };

    let user = db_conn.update_or_add_user(new_user).await;

    if config.whitelisted_emails.contains(&gh_email) && !user.active {
        db_conn.update_user(database::UpdateUser::default().id(user.id).active(true)).await;
    }

    let session_id =
        SessionId::User(UserSessionId { session_id: new_session_id(), user_id: user.id });

    let access_token = token_signer.create_access_header_value(&session_id).unwrap();
    let access_token = access_token.to_str().unwrap();
    let refresh_token = token_signer.create_refresh_header_value(&session_id).unwrap();
    let refresh_token = refresh_token.to_str().unwrap();

    let mut location = config.web_base_url.join("/login").unwrap();

    location
        .query_pairs_mut()
        .append_pair("access_token", access_token)
        .append_pair("refresh_token", refresh_token);

    Ok(HttpResponse::TemporaryRedirect().append_header((LOCATION, location.as_str())).finish())
}

#[derive(Debug, Error)]
pub enum AuthCodeError {
    #[error("Access is restricted by allowlist")]
    RestrictedByAllowlist,
}

impl ResponseError for AuthCodeError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            AuthCodeError::RestrictedByAllowlist => {
                HttpResponse::Forbidden().body("Access is restricted")
            }
        }
    }
}
