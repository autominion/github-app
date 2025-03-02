use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UserInfo {
    pub name: String,
    pub email_domain: String,
    pub active: bool,
    pub on_waitlist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Repo {
    pub id: String,
    pub name: String,
    pub active: bool,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    Member,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RepoUserInfo {
    pub id: String,
    pub name: String,
    pub github_login: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AddRepoUserRequest {
    pub github_login: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TaskInfo {
    pub id: String,
    pub repo_name: String,
    pub issue_number: i64,
    pub status: TaskStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TaskDetails {
    pub id: String,
    pub repo_name: String,
    pub issue_number: i64,
    pub status: TaskStatus,
    pub interactions: Vec<LLMInteraction>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LLMInteraction {
    pub id: String,
    pub request: Option<Value>,
    pub response: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TaskPollResponse {
    pub status: TaskStatus,
    pub interactions: Vec<LLMInteraction>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct OpenRouterStatus {
    pub connected: bool,
}
