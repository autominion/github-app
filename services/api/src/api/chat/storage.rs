use serde_json::Value;
use uuid::Uuid;

use database::{Conn, NewLLMInteraction};

/// Stores an interaction in the database by saving the serialized request and response.
///
/// For streaming requests the response is `None` at the moment.
pub async fn store_interaction(
    conn: &mut Conn<'_>,
    task_id: Uuid,
    request: &impl serde::Serialize,
    response: Option<Value>,
) {
    let request = serde_json::to_value(request).ok();
    let new_interaction = NewLLMInteraction { task_id, request, response };
    conn.add_llm_interaction(new_interaction).await;
}
