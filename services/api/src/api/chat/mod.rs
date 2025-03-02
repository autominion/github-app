use actix_web::{
    http::header,
    post,
    web::{self, Json},
    HttpResponse, Scope,
};
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use reqwest::Client;
use std::time::Duration;
use url::Url;

use auth::AgentSessionId;
use database::Database;

mod requests;
mod storage;

use requests::CompletionRequest;
use storage::store_interaction;

/// Base URL for the OpenRouter API.
static OPENROUTER_BASE_URL: Lazy<Url> = Lazy::new(|| {
    Url::parse("https://openrouter.ai/api/v1").expect("Failed to parse OpenRouter base URL")
});

const ROUNDTRIP_TIMEOUT: Duration = Duration::from_secs(5 * 60);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Create a `reqwest` client with lenient timeouts.
fn create_reqwest_client() -> Client {
    reqwest::Client::builder()
        .timeout(ROUNDTRIP_TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .build()
        .expect("Failed to build streaming HTTP client")
}

pub fn scope() -> Scope {
    Scope::new("/chat").service(completions)
}

#[post("/completions")]
pub async fn completions(
    agent: AgentSessionId,
    db: web::Data<Database>,
    body: Json<CompletionRequest>,
) -> HttpResponse {
    // Fetch the user’s API key.
    let task_id = agent.task_id;
    let mut conn = db.conn().await;
    let user_id = conn.user_id_that_created_task(&task_id).await;
    let user = conn.get_user(&user_id).await;
    let api_key = match user.openrouter_key {
        Some(k) => k,
        None => return HttpResponse::BadRequest().body("No OpenRouter API key configured."),
    };

    // Extract the request payload.
    let request_payload = body.into_inner();

    // Check if user requested streaming. If `stream` is `true`, we do SSE.
    if request_payload.stream.unwrap_or(false) {
        // For streaming requests, store the request with no response for now.
        store_interaction(&mut conn, task_id, &request_payload, None).await;
        forward_stream_request(&api_key, &request_payload).await
    } else {
        // For non-streaming requests, forward the request and capture the response.
        match forward_non_stream_request(&api_key, &request_payload).await {
            Ok((resp, response_json)) => {
                // Store both the request and its response.
                store_interaction(&mut conn, task_id, &request_payload, response_json).await;
                resp
            }
            Err(err_resp) => err_resp,
        }
    }
}

/// Forward non-streaming completions
async fn forward_non_stream_request(
    api_key: &str,
    request_payload: &CompletionRequest,
) -> Result<(HttpResponse, Option<serde_json::Value>), HttpResponse> {
    let client = create_reqwest_client();

    let url = format!("{}/chat/completions", OPENROUTER_BASE_URL.as_str());
    let req_builder = client
        .post(url)
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .json(&request_payload);

    let resp = req_builder.send().await.map_err(|err| {
        log::error!("Failed to send request to OpenRouter: {:?}", err);
        HttpResponse::InternalServerError().finish()
    })?;

    let status = resp.status();
    let text_body = resp.text().await.map_err(|err| {
        log::error!("Failed to read response body: {:?}", err);
        HttpResponse::InternalServerError().finish()
    })?;

    if status.is_success() {
        // Try to parse the response as JSON; if parsing fails, wrap the raw text.
        let response_json: Option<serde_json::Value> = serde_json::from_str(&text_body).ok();
        Ok((HttpResponse::Ok().body(text_body), response_json))
    } else if status.is_client_error() {
        Err(HttpResponse::BadRequest().body(text_body))
    } else {
        log::error!("OpenRouter error: status={} body={}", status, text_body);
        Err(HttpResponse::InternalServerError().finish())
    }
}

/// Forward streaming completions (SSE)
async fn forward_stream_request(
    api_key: &str,
    request_payload: &CompletionRequest,
) -> HttpResponse {
    let client = create_reqwest_client();

    let url = format!("{}/chat/completions", OPENROUTER_BASE_URL.as_str());
    let req_builder = client
        .post(url)
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .json(&request_payload);

    let resp = match req_builder.send().await {
        Ok(r) => r,
        Err(err) => {
            log::error!("Failed to send SSE request to OpenRouter: {:?}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let status = resp.status();
    if !status.is_success() {
        let text_body = match resp.text().await {
            Ok(b) => b,
            Err(e) => {
                log::error!("Failed to read SSE error body: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };
        return if status.is_client_error() {
            HttpResponse::BadRequest().body(text_body)
        } else {
            log::error!("OpenRouter SSE upstream error: status={} body={}", status, text_body);
            HttpResponse::InternalServerError().finish()
        };
    }

    // Stream the upstream bytes directly to the client
    let byte_stream = resp.bytes_stream().map(|chunk| match chunk {
        Ok(c) => Ok(c),
        Err(err) => {
            log::error!("Error reading SSE chunk: {:?}", err);
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    });

    // Return an SSE response
    HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "text/event-stream"))
        .append_header((header::CACHE_CONTROL, "no-cache"))
        .streaming(byte_stream)
}
