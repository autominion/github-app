use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::conn::Conn;
use crate::models::llm_interactions::{LLMInteraction, NewLLMInteraction};
use crate::schema::llm_interactions::dsl::*;

impl Conn<'_> {
    pub async fn add_llm_interaction(
        &mut self,
        new_interaction: NewLLMInteraction,
    ) -> LLMInteraction {
        diesel::insert_into(llm_interactions)
            .values(new_interaction)
            .get_result(&mut self.conn)
            .await
            .expect("Error inserting new interaction")
    }

    pub async fn llm_interactions(&mut self, the_task_id: &Uuid) -> Vec<LLMInteraction> {
        llm_interactions
            .filter(task_id.eq(the_task_id))
            .order_by(created_at)
            .load(&mut self.conn)
            .await
            .expect("Error loading interactions")
    }

    pub async fn llm_interactions_after(
        &mut self,
        the_task_id: Uuid,
        the_interaction_id: Uuid,
    ) -> Vec<LLMInteraction> {
        let the_interaction_created_at = llm_interactions
            .filter(id.eq(the_interaction_id))
            .select(created_at)
            .first::<chrono::NaiveDateTime>(&mut self.conn)
            .await
            .expect("Error getting interaction created_at");

        llm_interactions
            .filter(task_id.eq(the_task_id))
            .select(LLMInteraction::as_select())
            .filter(created_at.gt(the_interaction_created_at))
            .load(&mut self.conn)
            .await
            .expect("Error loading interactions")
    }
}
