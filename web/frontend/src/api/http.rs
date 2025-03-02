use std::fmt;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use url::Url;

use user_api::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ApiError {
    NotFound,
    Internal(String),
    Unauthorized,
    Forbidden,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound => write!(f, "Not found"),
            ApiError::Internal(msg) => write!(f, "Internal error: {}", msg),
            ApiError::Unauthorized => write!(f, "Unauthorized"),
            ApiError::Forbidden => write!(f, "Access forbidden"),
        }
    }
}

impl std::error::Error for ApiError {}

pub async fn user_info() -> Result<UserInfo, ApiError> {
    get_json("user/info").await
}

pub async fn openrouter_status() -> Result<OpenRouterStatus, ApiError> {
    get_json("user/openrouter").await
}

pub async fn disconnect_openrouter() -> Result<(), ApiError> {
    delete("user/openrouter").await
}

pub async fn repos() -> Result<Vec<Repo>, ApiError> {
    get_json("repos").await
}

pub async fn repo(id: &str) -> Result<Repo, ApiError> {
    get_json(&format!("repos/{}", id)).await
}

pub async fn add_repo(id: &str) -> Result<(), ApiError> {
    put(&format!("repos/{}/active", id)).await
}

pub async fn remove_repo(id: &str) -> Result<(), ApiError> {
    delete(&format!("repos/{}/active", id)).await
}

pub async fn repo_users(id: &str) -> Result<Vec<RepoUserInfo>, ApiError> {
    get_json(&format!("repos/{}/users", id)).await
}

pub async fn add_repo_user(id: &str, login: &str) -> Result<(), ApiError> {
    post_json(
        &format!("repos/{}/users", id),
        &AddRepoUserRequest { github_login: login.to_owned() },
    )
    .await
}

pub async fn delete_repo_users(repo_id: &str, user_id: &str) -> Result<(), ApiError> {
    delete(&format!("repos/{}/users/{}", repo_id, user_id)).await
}

pub async fn join_waitlist() -> Result<(), ApiError> {
    put("user/waitlist").await
}

pub async fn leave_waitlist() -> Result<(), ApiError> {
    delete("user/waitlist").await
}

pub async fn tasks() -> Result<Vec<TaskInfo>, ApiError> {
    get_json("tasks").await
}

pub async fn task(id: &str) -> Result<TaskDetails, ApiError> {
    get_json(&format!("tasks/{}", id)).await
}

pub async fn task_logs(id: &str) -> Result<String, ApiError> {
    get_raw_text(&format!("tasks/{}/logs", id)).await
}

/// Perform an HTTP GET and parses the response as JSON.
pub async fn get_json<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    send_request(reqwest::Method::GET, path, |b| b, |r| r.json::<T>()).await
}

/// Perform an HTTP GET and returns the raw text of the response.
pub async fn get_raw_text(path: &str) -> Result<String, ApiError> {
    send_request(reqwest::Method::GET, path, |b| b, |r| r.text()).await
}

/// Perform an HTTP PUT without any request body. Returns an empty result on success.
pub async fn put(path: &str) -> Result<(), ApiError> {
    send_request(reqwest::Method::PUT, path, |b| b, |_| async { Ok(()) }).await
}

/// Perform an HTTP DELETE. Returns an empty result on success.
pub async fn delete(path: &str) -> Result<(), ApiError> {
    send_request(reqwest::Method::DELETE, path, |b| b, |_| async { Ok(()) }).await
}

/// Perform an HTTP POST with a JSON body. Returns an empty result on success.
pub async fn post_json<T: Serialize>(path: &str, body: T) -> Result<(), ApiError> {
    send_request(reqwest::Method::POST, path, |b| b.json(&body), |_| async { Ok(()) }).await
}

/// Send a request with the given HTTP method and path.
/// The `configure` closure allows further modification of the request (for example,
/// adding a JSON body), and the `extractor` closure is used to parse the response.
async fn send_request<T, F, Fut>(
    method: reqwest::Method,
    path: &str,
    configure: impl FnOnce(reqwest::RequestBuilder) -> reqwest::RequestBuilder,
    extractor: F,
) -> Result<T, ApiError>
where
    F: FnOnce(reqwest::Response) -> Fut,
    Fut: std::future::Future<Output = Result<T, reqwest::Error>>,
{
    let url = build_url(path)?;
    let builder = reqwest::Client::new().request(method, url);
    let builder = configure(builder);
    let response = builder.send().await.map_err(|e| ApiError::Internal(e.to_string()))?;
    process_response(response, extractor).await
}

/// Construct the full URL for a given API path. The base is derived from the
/// current window location with `/api/` appended.
fn build_url(path: &str) -> Result<Url, ApiError> {
    let window =
        web_sys::window().ok_or_else(|| ApiError::Internal("Window object not found".into()))?;
    let href = window
        .location()
        .href()
        .map_err(|_| ApiError::Internal("Failed to retrieve window location href".into()))?;
    let base_url = Url::parse(&href).map_err(|e| ApiError::Internal(e.to_string()))?;
    let api_base = base_url.join("/api/").map_err(|e| ApiError::Internal(e.to_string()))?;
    api_base.join(path).map_err(|e| ApiError::Internal(e.to_string()))
}

/// Process an HTTP response:
/// - If the status is a success, the provided `extractor` function is used to parse the response.
/// - Otherwise, the status code is mapped to an appropriate `ApiError`.
async fn process_response<T, F, Fut>(
    response: reqwest::Response,
    extractor: F,
) -> Result<T, ApiError>
where
    F: FnOnce(reqwest::Response) -> Fut,
    Fut: std::future::Future<Output = Result<T, reqwest::Error>>,
{
    let status = response.status();
    if status.is_success() {
        extractor(response).await.map_err(|e| ApiError::Internal(e.to_string()))
    } else if status == reqwest::StatusCode::NOT_FOUND {
        Err(ApiError::NotFound)
    } else if status == reqwest::StatusCode::UNAUTHORIZED {
        Err(ApiError::Unauthorized)
    } else if status == reqwest::StatusCode::FORBIDDEN {
        Err(ApiError::Forbidden)
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".into());
        Err(ApiError::Internal(format!("HTTP {}: {}", status, error_text)))
    }
}
