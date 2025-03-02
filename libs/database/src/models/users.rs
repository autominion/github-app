use chrono::{DateTime, Utc};
use diesel::AsChangeset;
use diesel::Identifiable;
use diesel::Selectable;
use diesel::{Insertable, Queryable};
use uuid::Uuid;

use crate::schema::users;
use crate::Update;

#[derive(Queryable, Identifiable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub active: bool,
    pub joined_waitlist_at: Option<DateTime<Utc>>,
    pub github_id: String,
    pub github_email: Option<String>,
    pub github_name: Option<String>,
    pub github_login: String,
    pub github_access_token: Option<String>,
    pub github_access_token_expires_at: Option<DateTime<Utc>>,
    pub openrouter_key: Option<String>,
    pub openrouter_code_verifier: Option<String>,
}

impl Update for User {
    type Output = UpdateUser;

    fn update(&self) -> Self::Output {
        UpdateUser { id: self.id, ..Default::default() }
    }
}

#[derive(Default, AsChangeset, Identifiable)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    id: Uuid,
    active: Option<bool>,
    joined_waitlist_at: Option<Option<DateTime<Utc>>>,
    github_email: Option<Option<String>>,
    github_name: Option<Option<String>>,
    github_login: Option<String>,
    github_access_token: Option<Option<String>>,
    github_access_token_expires_at: Option<Option<DateTime<Utc>>>,
    openrouter_key: Option<Option<String>>,
    openrouter_code_verifier: Option<Option<String>>,
}

impl UpdateUser {
    pub fn id(mut self, id: Uuid) -> UpdateUser {
        self.id = id;
        self
    }

    pub fn active(mut self, active: bool) -> UpdateUser {
        self.active = Some(active);
        self
    }

    pub fn joined_waitlist_at(mut self, joined_waitlist_at: Option<DateTime<Utc>>) -> UpdateUser {
        self.joined_waitlist_at = Some(joined_waitlist_at);
        self
    }

    pub fn github_email(mut self, github_email: Option<String>) -> UpdateUser {
        self.github_email = Some(github_email);
        self
    }

    pub fn github_name(mut self, github_name: Option<String>) -> UpdateUser {
        self.github_name = Some(github_name);
        self
    }

    pub fn github_login(mut self, github_login: String) -> UpdateUser {
        self.github_login = Some(github_login);
        self
    }

    pub fn github_access_token(mut self, github_access_token: Option<String>) -> UpdateUser {
        self.github_access_token = Some(github_access_token);
        self
    }

    pub fn github_access_token_expires_at(
        mut self,
        github_access_token_expires_at: Option<DateTime<Utc>>,
    ) -> UpdateUser {
        self.github_access_token_expires_at = Some(github_access_token_expires_at);
        self
    }

    pub fn openrouter_key(mut self, openrouter_key: Option<String>) -> UpdateUser {
        self.openrouter_key = Some(openrouter_key);
        self
    }

    pub fn openrouter_code_verifier(
        mut self,
        openrouter_code_verifier: Option<String>,
    ) -> UpdateUser {
        self.openrouter_code_verifier = Some(openrouter_code_verifier);
        self
    }
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub github_id: String,
    pub github_email: Option<Option<String>>,
    pub github_name: Option<Option<String>>,
    pub github_login: String,
    pub github_access_token: Option<Option<String>>,
    pub github_access_token_expires_at: Option<Option<DateTime<Utc>>>,
}
