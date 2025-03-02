use std::future::Future;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use user_api::{OpenRouterStatus, Repo, RepoUserInfo, TaskDetails, TaskInfo, UserInfo};

use crate::api::http;
use crate::api::http::ApiError;
use crate::errors::{handle_api_result, ErrorStore};

/// A generic API resource that creates a Leptos resource from an async fetcher.
/// On an Unauthorized error it navigates to the login page; on any other error,
/// the error is sent to a global error store (which must be provided via context).
pub fn use_api<T, F, Fut>(fetch: F) -> LocalResource<Result<T, ApiError>>
where
    T: Clone + Serialize + DeserializeOwned + 'static,
    F: Fn() -> Fut + 'static,
    Fut: Future<Output = Result<T, ApiError>> + 'static,
{
    let navigate = Arc::new(use_navigate());
    let fetch = Arc::new(fetch);
    let error_store = expect_context::<RwSignal<ErrorStore>>();

    LocalResource::new(move || {
        let navigate = navigate.clone();
        let fetch = fetch.clone();
        async move {
            let result = fetch().await;
            handle_api_result(result, navigate.clone(), &error_store)
        }
    })
}

/// Fetches the user info.
pub fn use_user_info() -> LocalResource<Result<UserInfo, ApiError>> {
    use_api(|| async { http::user_info().await })
}

/// Fetches the OpenRouter status.
pub fn use_openrouter_status() -> LocalResource<Result<OpenRouterStatus, ApiError>> {
    use_api(|| async { http::openrouter_status().await })
}

/// Fetches the repositories.
pub fn use_repos() -> LocalResource<Result<Vec<Repo>, ApiError>> {
    use_api(|| async { http::repos().await })
}

/// Fetches details for a single task.
pub fn use_task(id: impl ToString) -> LocalResource<Result<TaskDetails, ApiError>> {
    let id = id.to_string();
    use_api(move || {
        let id = id.clone();
        async move { http::task(&id).await }
    })
}

/// Fetches the logs for a task.
pub fn use_task_logs(id: impl ToString) -> LocalResource<Result<Option<String>, ApiError>> {
    let id = id.to_string();
    use_api(move || {
        let id = id.clone();
        async move {
            match http::task_logs(&id).await {
                Ok(logs) => Ok(Some(logs)),
                Err(ApiError::NotFound) => Ok(None),
                Err(err) => Err(err),
            }
        }
    })
}

/// Fetches the list of tasks.
pub fn use_tasks() -> LocalResource<Result<Vec<TaskInfo>, ApiError>> {
    use_api(|| async { http::tasks().await })
}

/// Fetches a repository by id.
pub fn use_repo(id: impl ToString) -> LocalResource<Result<Repo, ApiError>> {
    let id = id.to_string();
    use_api(move || {
        let id = id.clone();
        async move { http::repo(&id).await }
    })
}

/// Fetches the repository’s users.
pub fn use_repo_users(
    repo_id: impl ToString,
) -> LocalResource<Result<Vec<RepoUserInfo>, ApiError>> {
    let repo_id = repo_id.to_string();
    use_api(move || {
        let repo_id = repo_id.clone();
        async move { http::repo_users(&repo_id).await }
    })
}
