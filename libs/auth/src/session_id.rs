use std::future::{ready, Future};
use std::pin::Pin;
use std::time::Duration;
use std::{fmt, future::Ready};

use actix_web::HttpMessage;
use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use jwt_compact::alg::{Ed25519, SigningKey};
use rand::{distributions::Alphanumeric, rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TokenSigner = actix_jwt_auth_middleware::TokenSigner<SessionId, Ed25519>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserSessionId {
    pub session_id: String,
    pub user_id: Uuid,
}

impl fmt::Display for UserSessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.session_id, self.user_id)
    }
}

pub fn token_signer(config: &config::Config) -> TokenSigner {
    let jwt_private_key = config.jwt_expanded_private_key.clone();
    let secret_key = move || SigningKey::from_slice(&jwt_private_key).unwrap();

    TokenSigner::new()
        .signing_key(secret_key())
        .algorithm(Ed25519)
        .refresh_token_lifetime(Duration::from_secs(60 * 60))
        .build()
        .expect("Failed to create TokenSigner")
}

pub fn issue_agent_token(
    token_signer: &TokenSigner,
    task_id: &Uuid,
    token_lifetime: Duration,
) -> String {
    let session_id = new_session_id();
    let agent_session_id: SessionId = AgentSessionId { session_id, task_id: *task_id }.into();
    token_signer.create_signed_token(&agent_session_id, token_lifetime).unwrap()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentSessionId {
    pub session_id: String,
    pub task_id: Uuid,
}

impl fmt::Display for AgentSessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.session_id, self.task_id)
    }
}

// Create the SessionId enum with serde tagging
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum SessionId {
    User(UserSessionId),
    Agent(AgentSessionId),
}

impl From<UserSessionId> for SessionId {
    fn from(user_session_id: UserSessionId) -> Self {
        SessionId::User(user_session_id)
    }
}

impl From<AgentSessionId> for SessionId {
    fn from(agent_session_id: AgentSessionId) -> Self {
        SessionId::Agent(agent_session_id)
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionId::User(user_session_id) => user_session_id.fmt(f),
            SessionId::Agent(agent_session_id) => agent_session_id.fmt(f),
        }
    }
}

impl FromRequest for SessionId {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(session_id) = req.extensions().get::<SessionId>() {
            ready(Ok(session_id.clone()))
        } else {
            ready(Err(actix_web::error::ErrorUnauthorized("Unauthorized")))
        }
    }
}

impl FromRequest for UserSessionId {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = SessionId::from_request(req, payload);
        Box::pin(async move {
            match fut.await? {
                SessionId::User(user_session_id) => Ok(user_session_id),
                _ => Err(actix_web::error::ErrorUnauthorized("Unauthorized")),
            }
        })
    }
}

impl FromRequest for AgentSessionId {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = SessionId::from_request(req, payload);
        Box::pin(async move {
            match fut.await? {
                SessionId::Agent(agent_session_id) => Ok(agent_session_id),
                _ => Err(actix_web::error::ErrorUnauthorized("Unauthorized")),
            }
        })
    }
}

const SESSION_ID_LEN: usize = 32;

pub fn new_session_id() -> String {
    StdRng::from_entropy().sample_iter(&Alphanumeric).take(SESSION_ID_LEN).map(char::from).collect()
}
