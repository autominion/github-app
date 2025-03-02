use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::conn::Conn;
use crate::models::agent_configs::{AgentConfig, NewAgentConfig, UpdateAgentConfig};
use crate::schema::agent_configs::dsl::*;

impl Conn<'_> {
    /// Add a new `AgentConfig` to the database.
    pub async fn add_agent_config(&mut self, new_agent_config: NewAgentConfig) -> AgentConfig {
        diesel::insert_into(agent_configs)
            .values(new_agent_config)
            .get_result(&mut self.conn)
            .await
            .unwrap()
    }

    /// Update an existing `AgentConfig` in the database.
    pub async fn update_agent_config(&mut self, update: UpdateAgentConfig) -> AgentConfig {
        diesel::update(&update).set(&update).get_result(&mut self.conn).await.unwrap()
    }

    /// Retrieve an `AgentConfig` by its ID.
    pub async fn get_agent_config(&mut self, agent_config_id: &Uuid) -> AgentConfig {
        agent_configs.filter(id.eq(agent_config_id)).get_result(&mut self.conn).await.unwrap()
    }

    /// Delete an `AgentConfig` by its ID.
    pub async fn delete_agent_config(&mut self, agent_config_id: &Uuid) -> usize {
        diesel::delete(agent_configs.filter(id.eq(agent_config_id)))
            .execute(&mut self.conn)
            .await
            .unwrap()
    }
}
