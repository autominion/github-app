use chrono::{DateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use serde_json::Value;
use uuid::Uuid;

use crate::schema::llm_interactions;

#[derive(Queryable, Identifiable, Selectable)]
#[diesel(belongs_to(Task))]
#[diesel(table_name = llm_interactions)]
pub struct LLMInteraction {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub task_id: Uuid,
    pub request: Option<Value>,
    pub response: Option<Value>,
}

#[derive(Insertable)]
#[diesel(table_name = llm_interactions)]
pub struct NewLLMInteraction {
    pub task_id: Uuid,
    pub request: Option<Value>,
    pub response: Option<Value>,
}
