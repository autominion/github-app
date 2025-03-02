use actix_web::{get, http::header::LOCATION, web, HttpResponse};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use once_cell::sync::Lazy;
use rand::RngCore as _;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;
use uuid::Uuid;

use auth::UserSessionId;
use config::Config;
use database::{Database, UpdateUser};

pub static OAUTH_AUTHORIZE_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://openrouter.ai/auth").unwrap());

pub static AUTH_KEY_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://openrouter.ai/api/v1/auth/keys").unwrap());

#[get("/auth/openrouter")]
async fn openrouter_connect(
    session_id: UserSessionId,
    config: web::Data<Config>,
    db: web::Data<Database>,
) -> HttpResponse {
    let code_verifier = code_verifier();
    {
        let mut conn = db.conn().await;
        conn.update_user(
            UpdateUser::default()
                .id(session_id.user_id)
                .openrouter_code_verifier(Some(code_verifier.clone())),
        )
        .await;
    }
    let code_challenge = code_challenge(&code_verifier);

    let mut location = OAUTH_AUTHORIZE_URL.clone();
    let callback_url = config
        .web_base_url
        .join(&format!("/auth/openrouter/auth-code/{}", session_id.user_id))
        .unwrap();
    location
        .query_pairs_mut()
        .append_pair("callback_url", callback_url.as_str())
        .append_pair("code_challenge", code_challenge.as_str())
        .append_pair("code_challenge_method", "S256");
    HttpResponse::TemporaryRedirect().append_header((LOCATION, location.as_str())).finish()
}

#[derive(Deserialize)]
struct AuthCodeQuery {
    code: String,
}

#[get("/auth/openrouter/auth-code/{user_id}")]
async fn openrouter_auth_code(
    config: web::Data<Config>,
    db: web::Data<Database>,
    user_id: web::Path<Uuid>,
    query: web::Query<AuthCodeQuery>,
) -> HttpResponse {
    let user_id = user_id.into_inner();

    let code_verifier = {
        let mut conn = db.conn().await;
        let user = conn.get_user(&user_id).await;
        match user.openrouter_code_verifier {
            Some(code_verifier) => code_verifier,
            None => return HttpResponse::BadRequest().finish(),
        }
    };

    let key = auth_key(&query.code, &code_verifier).await.unwrap();
    {
        let mut conn = db.conn().await;
        conn.update_user(
            UpdateUser::default()
                .id(user_id)
                .openrouter_code_verifier(None)
                .openrouter_key(Some(key)),
        )
        .await;
    }

    let location = config.web_base_url.join("/settings").unwrap();
    HttpResponse::TemporaryRedirect().append_header((LOCATION, location.as_str())).finish()
}

pub async fn auth_key(code: &str, code_verifier: &str) -> reqwest::Result<String> {
    let request = AuthKeyRequest { code, code_verifier, code_challenge_method: "S256" };
    let response = reqwest::Client::new().post(AUTH_KEY_URL.as_str()).json(&request).send().await?;
    let response = response.error_for_status()?;
    let response = response.json::<AuthKeyResponse>().await?;
    Ok(response.key)
}

#[derive(Serialize)]
struct AuthKeyRequest<'a> {
    code: &'a str,
    code_verifier: &'a str,
    code_challenge_method: &'a str,
}

#[derive(Deserialize)]
struct AuthKeyResponse {
    key: String,
}

/// Generate a random PKCE code verifier of length exactly 128 Base64 characters (no padding).
pub fn code_verifier() -> String {
    // 96 raw bytes => 128 Base64URL characters, given 3 bytes => 4 Base64 chars.
    let mut verifier_bytes = [0u8; 96];
    rand::thread_rng().fill_bytes(&mut verifier_bytes);
    URL_SAFE_NO_PAD.encode(verifier_bytes)
}

/// Compute the PKCE code challenge from the code verifier
/// as a Base64-url-encoded (no padding) SHA-256 hash.
pub fn code_challenge(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}
