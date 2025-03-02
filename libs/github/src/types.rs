use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub login: String,
    pub name: Option<String>,
}
pub struct IssueInfo {
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub name: Option<String>,
    pub login: String,
    pub node_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Membership {
    pub role: Role,
}

#[derive(Debug, Deserialize)]
pub enum Role {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "member")]
    Member,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct Repo {
    pub full_name: String,
    pub name: String,
    pub node_id: String,
    pub private: bool,
}

#[derive(Debug, Deserialize)]
pub struct Issue {
    pub node_id: String,
    pub title: String,
    pub body: String,
    pub number: i64,
    pub user: Option<User>,
}

#[derive(Debug, Deserialize)]
pub struct Comment {
    pub node_id: String,
    pub body: String,
    pub user: User,
}

/// An installation of a GitHub App
#[derive(Debug, Deserialize)]
pub struct Installation {
    pub id: i64,
    pub app_id: i64,
    pub target_id: i64,
    pub target_type: String,
    pub events: Vec<String>,
    pub account: Account,
}

#[derive(Debug, Deserialize)]
pub struct Account {
    pub login: String,
    pub id: i64,
    pub node_id: String,
    pub r#type: AccountType,
}

/// GitHub account type
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum AccountType {
    User,
    Organization,
    /// Fallback variant for unrecognized/undocumented account types
    #[serde(other)]
    Unknown,
}

/// https://docs.github.com/en/rest/apps/installations?apiVersion=2022-11-28#list-repositories-accessible-to-the-app-installation
#[derive(Deserialize)]
pub struct InstallationRepositories {
    pub repositories: Vec<InstallationRepository>,
}

#[derive(Deserialize)]
pub struct InstallationRepository {
    pub id: i64,
    pub node_id: String,
    pub name: String,
    pub full_name: String,
    pub private: bool,
}
