use std::sync::Arc;

use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorUnauthorized;
use actix_web::{Error, HttpMessage};
use actix_web_httpauth::extractors::basic::BasicAuth;
use ed25519_compact::PublicKey;
use github::GitHub;
use jwt_compact::{alg::Ed25519, Token};
use jwt_compact::{AlgorithmExt, UntrustedToken};

use auth::{AgentSessionId, SessionId};
use database::Database;

use git_proxy::{ForwardToRemote, ProxyBehaivor};

pub async fn basic_auth_validator(
    db: Arc<Database>,
    github: Arc<GitHub>,
    public_key: Arc<PublicKey>,
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let Some(agent_token) = credentials.password() else {
        return Err((ErrorUnauthorized("Invalid username or password"), req));
    };

    let Ok(token) = extract_session_jwt(agent_token, &public_key) else {
        return Err((ErrorUnauthorized("Invalid username or password"), req));
    };

    let session_id = &token.claims().custom;

    let Ok(session_id) = expect_agent_session(session_id) else {
        return Err((ErrorUnauthorized("Expected agent session"), req));
    };

    let task_id = session_id.task_id;

    let mut conn = db.conn().await;
    let (task, repo) = conn.get_task_and_repository(&task_id).await;

    let access_token = github.installation_access_token(&github.github_app_jwt()).await;
    let github_inst = github.with_access(&access_token.token);

    let numeric_repo_id = github_inst.repo_numeric_id_by_node_id(&repo.github_id).await;

    let github_access_token =
        github.create_scoped_access_token(&github.github_app_jwt(), numeric_repo_id).await;

    let raw_repo_url = format!("https://github.com/{}", repo.github_full_name);
    let url = raw_repo_url.parse().expect("Expected valid repo url");
    // For now we assume that the task id is the branch name
    let allowed_ref = format!("refs/heads/{}", task.id);

    req.extensions_mut().insert(ProxyBehaivor::ForwardToRemote(ForwardToRemote {
        url,
        basic_auth_user: "x-access-token".to_string(),
        basic_auth_pass: github_access_token.token,
        allowed_ref,
    }));

    Ok(req)
}

fn extract_session_jwt(
    token_str: &str,
    public_key: &PublicKey,
) -> Result<Token<SessionId>, Box<dyn std::error::Error>> {
    let token = UntrustedToken::new(&token_str)?;
    let token = Ed25519.validator(public_key).validate(&token)?;

    Ok(token)
}

fn expect_agent_session(session_id: &SessionId) -> Result<AgentSessionId, &str> {
    match session_id {
        SessionId::Agent(agent_session_id) => Ok(agent_session_id.clone()),
        _ => Err("Invalid session id type"),
    }
}
