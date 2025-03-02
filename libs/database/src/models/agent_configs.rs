use chrono::{DateTime, Utc};
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::schema::agent_configs;
use crate::Update;

#[derive(Queryable, Identifiable, Selectable)]
#[diesel(table_name = agent_configs)]
pub struct AgentConfig {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub container_registry_host: String,
    pub container_registry_username: Option<String>,
    pub container_registry_password: Option<String>,
    pub container_image: String,
}

impl Update for AgentConfig {
    type Output = UpdateAgentConfig;

    fn update(&self) -> Self::Output {
        UpdateAgentConfig { id: self.id, ..Default::default() }
    }
}

#[derive(Default, AsChangeset, Identifiable)]
#[diesel(table_name = agent_configs)]
pub struct UpdateAgentConfig {
    pub id: Uuid,
    pub container_registry_host: Option<String>,
    pub container_registry_username: Option<Option<String>>,
    pub container_registry_password: Option<Option<String>>,
    pub container_image: Option<String>,
}

impl UpdateAgentConfig {
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn container_registry_host(mut self, url: String) -> Self {
        self.container_registry_host = Some(url);
        self
    }

    pub fn container_registry_username(mut self, username: Option<String>) -> Self {
        self.container_registry_username = Some(username);
        self
    }

    pub fn container_registry_password(mut self, password: Option<String>) -> Self {
        self.container_registry_password = Some(password);
        self
    }

    pub fn container_image(mut self, image: String) -> Self {
        self.container_image = Some(image);
        self
    }
}

#[derive(Insertable)]
#[diesel(table_name = agent_configs)]
pub struct NewAgentConfig {
    pub container_registry_host: String,
    pub container_registry_username: Option<String>,
    pub container_registry_password: Option<String>,
    pub container_image: String,
}

impl NewAgentConfig {
    pub fn into_update(self, id: Uuid) -> UpdateAgentConfig {
        let NewAgentConfig {
            container_registry_host,
            container_registry_username,
            container_registry_password,
            container_image,
        } = self;

        let mut update_agent_config = UpdateAgentConfig::default()
            .id(id)
            .container_registry_host(container_registry_host)
            .container_image(container_image);

        update_agent_config =
            update_agent_config.container_registry_username(container_registry_username);
        update_agent_config =
            update_agent_config.container_registry_password(container_registry_password);

        update_agent_config
    }
}
