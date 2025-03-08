use actix_web::{dev::Payload, web, Error, FromRequest, HttpRequest, Scope};
use serde_json::Value;
use url::Url;

use auth::AgentSessionId;
use database::Database;
use llm_proxy::{CompletionRequest, ProxyConfig};

use once_cell::sync::Lazy;

mod storage;

use storage::store_interaction;

static OPENROUTER_CHAT_COMPLETIONS_URL: Lazy<Url> = Lazy::new(|| {
    Url::parse("https://openrouter.ai/api/v1/chat/completions")
        .expect("Failed to parse OpenRouter chat completions URL")
});

pub fn scope() -> Scope {
    llm_proxy::scope(TheProxyConfig {})
}

#[derive(Clone)]
struct ProxyContext {
    agent_session: AgentSessionId,
    db: web::Data<Database>,
}

#[derive(Clone)]
struct TheProxyConfig {}

impl ProxyConfig for TheProxyConfig {
    type Context = ProxyContext;

    async fn extract_context(&self, req: &HttpRequest) -> Result<Self::Context, Error> {
        let agent_session = AgentSessionId::from_request(req, &mut Payload::None).await?;
        let db = req.app_data::<web::Data<Database>>().expect("Database not available").clone();
        Ok(ProxyContext { agent_session, db })
    }

    async fn api_key(
        &self,
        ctx: &Self::Context,
        _req: &CompletionRequest,
    ) -> Result<String, Error> {
        let mut conn = ctx.db.conn().await;
        let user_id = conn.user_id_that_created_task(&ctx.agent_session.task_id).await;
        let user = conn.get_user(&user_id).await;
        user.openrouter_key
            .ok_or(actix_web::error::ErrorBadRequest("No OpenRouter API key configured."))
    }

    async fn forward_to_url(
        &self,
        _ctx: &Self::Context,
        _req: &CompletionRequest,
    ) -> Result<Url, Error> {
        Ok(OPENROUTER_CHAT_COMPLETIONS_URL.clone())
    }

    async fn inspect_interaction(
        &self,
        ctx: &Self::Context,
        request: &CompletionRequest,
        response: Option<Value>,
    ) {
        let mut conn = ctx.db.conn().await;
        store_interaction(&mut conn, ctx.agent_session.task_id, request, response).await;
    }
}
