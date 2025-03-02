use graphql_client::GraphQLQuery;
use jwt_compact::alg::Rsa;
use jwt_compact::AlgorithmExt;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};
use url::Url;

use config::Config;

use crate::types::*;
use crate::util::unix_time_in_seconds;
use crate::IssueInfo;

use super::graphql::{add_comment, AddComment};
use super::graphql::{create_pull_request, CreatePullRequest};
use super::graphql::{create_repo, CreateRepo};
use super::graphql::{issue_id_view, IssueIdView};
use super::graphql::{issue_view, IssueView};
use super::graphql::{repo_numeric_id, RepoNumericId};
use super::graphql::{user_info_view, UserInfoView};
use super::graphql::{viewer_info, ViewerInfo};
use super::urls::{OAUTH_ACCESS_TOKEN_URL, REST_API_URL};

/// Accept header value
const GITHUB_ACCEPT_JSON: &str = "application/vnd.github+json";
const GITHUB_API_VERSION: &str = "2022-11-28";

#[derive(Clone)]
pub struct GitHub {
    client: reqwest::Client,
    config: config::Config,
}

impl GitHub {
    pub fn new(config: Config) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, GITHUB_ACCEPT_JSON.parse().unwrap());
        headers.insert("X-GitHub-Api-Version", GITHUB_API_VERSION.parse().unwrap());

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .user_agent(&config.github_api_user_agent)
            .build()
            .unwrap();

        Self { client, config }
    }

    pub fn with_access(&self, access_token: &str) -> WithAccess {
        WithAccess {
            config: self.config.clone(),
            client: self.client.clone(),
            access_token: access_token.to_owned(),
        }
    }

    /// Token to authenticate the GitHub app as a user
    /// https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app
    pub async fn user_access_token(&self, code: &str) -> UserAccessToken {
        self.client
            .post(OAUTH_ACCESS_TOKEN_URL.as_str())
            .query(&[
                ("client_id", self.config.github_app_client_id.as_str()),
                ("client_secret", self.config.github_app_client_secret.as_str()),
                ("code", code),
            ])
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    /// JWT access tokens for the GitHub app
    /// https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-json-web-token-jwt-for-a-github-app
    pub fn github_app_jwt(&self) -> AppJWT {
        let signing_key =
            RsaPrivateKey::from_pkcs1_pem(&self.config.github_app_private_key).unwrap();
        signing_key.validate().unwrap();
        let header: jwt_compact::Header<jwt_compact::Empty> = jwt_compact::Header::default();
        let curr_time = unix_time_in_seconds();
        let claims = jwt_compact::Claims::new(JWTClaims {
            // Issued at time, 60 seconds in the past to allow for clock drift
            iat: curr_time - 60,
            // JWT expiration time (10 minutes maximum)
            exp: curr_time + 10 * 60,
            // ID of the GitHub app
            iss: self.config.github_app_id.clone(),
            // At the moment, only RS256 is supported
            alg: "RS256".to_owned(),
        });

        let token = Rsa::rs256().token(&header, &claims, &signing_key).unwrap();
        AppJWT { token }
    }

    pub async fn set_webhook_config(&self, jwt: &AppJWT, url: Url) {
        let json = WebhookConfig {
            url,
            content_type: "json".to_owned(),
            secret: "".to_owned(),
            insecure_ssl: "0".to_owned(),
        };
        self.client
            .patch(REST_API_URL.join("/app/hook/config").unwrap())
            .header(AUTHORIZATION, jwt.header_value())
            .json(&json)
            .send()
            .await
            .unwrap();
    }

    /// Create a installation access token for the GitHub app
    pub async fn installation_access_token(&self, jwt: &AppJWT) -> InstallationAccessToken {
        let installation_id = &self.config.github_app_installation_id;
        let url = REST_API_URL
            .join(&format!("/app/installations/{installation_id}/access_tokens"))
            .unwrap();
        self.client
            .post(url)
            .header(AUTHORIZATION, jwt.header_value())
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    pub async fn create_scoped_access_token(
        &self,
        jwt: &AppJWT,
        repository_id: i64,
    ) -> InstallationAccessToken {
        let installation_id = &self.config.github_app_installation_id;
        let url = REST_API_URL
            .join(&format!("/app/installations/{installation_id}/access_tokens"))
            .unwrap();

        let request_body = ScopedAccessTokenRequest {
            repository_ids: vec![repository_id],
            permissions: Permissions { contents: "write".to_owned() },
        };

        self.client
            .post(url)
            .header(AUTHORIZATION, jwt.header_value())
            .json(&request_body)
            .send()
            .await
            .expect("Failed to send scoped access token request")
            .json()
            .await
            .expect("Failed to parse scoped access token response")
    }

    pub async fn installations(&self, jwt: &AppJWT) -> Vec<Installation> {
        let url = REST_API_URL.join("/app/installations").unwrap();
        self.client
            .get(url)
            .header(AUTHORIZATION, jwt.header_value())
            .send()
            .await
            .expect("Failed to fetch app installations")
            .json()
            .await
            .expect("Failed to parse app installations response")
    }
}

#[derive(Serialize)]
struct ScopedAccessTokenRequest {
    repository_ids: Vec<i64>,
    permissions: Permissions,
}

/// Represents the permissions granted to the access token.
#[derive(Serialize)]
struct Permissions {
    contents: String,
}

pub struct WithAccess {
    config: Config,
    client: reqwest::Client,
    access_token: String,
}

impl WithAccess {
    pub async fn viewer_info(&self) -> UserInfo {
        let response_data = self.graphql::<ViewerInfo>(viewer_info::Variables).await;

        UserInfo {
            id: response_data.viewer.id,
            login: response_data.viewer.login,
            name: response_data.viewer.name,
        }
    }

    pub async fn user_info(&self, login: &str) -> Option<UserInfo> {
        let vars = user_info_view::Variables { login: login.to_owned() };
        let response_data = self.graphql::<UserInfoView>(vars).await;

        response_data.user.map(|user| UserInfo { id: user.id, login: user.login, name: user.name })
    }

    pub async fn user_email(&self) -> String {
        let url = REST_API_URL.join("/user/emails").unwrap();
        let user_emails: Vec<UserEmail> = self
            .client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        user_emails
            .into_iter()
            .filter(|email| email.primary)
            .map(|email| email.email)
            .next()
            .unwrap()
    }

    pub async fn issue_info(&self, issue_id: &str) -> IssueInfo {
        let vars = issue_view::Variables { issue_id: issue_id.to_string() };
        let response_data = self.graphql::<IssueView>(vars).await;
        let issue_view::IssueViewNode::Issue(issue) = response_data.node.unwrap() else {
            panic!("expected issue");
        };
        IssueInfo { body: issue.body }
    }

    pub async fn issue_id(&self, repo_owner: &str, repo_name: &str, issue_number: i64) -> String {
        let vars = issue_id_view::Variables {
            repo_owner: repo_owner.to_owned(),
            repo_name: repo_name.to_owned(),
            issue_number,
        };
        let response_data = self.graphql::<IssueIdView>(vars).await;
        response_data.repository.unwrap().issue.unwrap().id
    }

    pub async fn add_comment(&self, subject_id: &str, body: &str) {
        let vars =
            add_comment::Variables { subject_id: subject_id.to_owned(), body: body.to_owned() };
        self.graphql::<AddComment>(vars).await;
    }

    pub async fn create_repo(&self, name: &str) -> String {
        let vars = create_repo::Variables {
            owner_id: self.config.github_app_organization_id.to_owned(),
            name: name.to_owned(),
        };
        let response_data = self.graphql::<CreateRepo>(vars).await;
        response_data.create_repository.unwrap().repository.unwrap().id
    }

    pub async fn delete_repo(&self, repo_name: &str) {
        let url = REST_API_URL.join(&format!("/repos/{}", repo_name)).unwrap();
        self.client
            .delete(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
            .send()
            .await
            .unwrap();
    }

    pub async fn create_pull_request(
        &self,
        repo_id: &str,
        title: &str,
        body: &str,
        head: &str,
    ) -> String {
        let vars = create_pull_request::Variables {
            repo_id: repo_id.to_owned(),
            title: title.to_owned(),
            body: body.to_owned(),
            head_ref: head.to_owned(),
            base_ref: "main".to_owned(),
        };
        let response_data = self.graphql::<CreatePullRequest>(vars).await;
        response_data.create_pull_request.unwrap().pull_request.unwrap().id
    }

    pub async fn repo_numeric_id_by_node_id(&self, node_id: &str) -> i64 {
        let vars = repo_numeric_id::Variables { node_id: node_id.to_owned() };
        let response_data = self.graphql::<RepoNumericId>(vars).await;

        let repo_numeric_id::RepoNumericIdNode::Repository(repo) = response_data.node.unwrap()
        else {
            panic!("expected repository");
        };
        repo.database_id.unwrap()
    }

    pub async fn installation_repositories(&self) -> Vec<InstallationRepository> {
        let url = REST_API_URL.join("/installation/repositories").unwrap();
        let InstallationRepositories { repositories } = self
            .client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        repositories
    }

    pub async fn organization_members(&self, org: &str) -> Vec<User> {
        let url = REST_API_URL.join(&format!("/orgs/{}/members", org)).unwrap();
        let members: Vec<User> = self
            .client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        members
    }

    pub async fn organization_membership(&self, org: &str, user: &str) -> Membership {
        let url = REST_API_URL.join(&format!("/orgs/{}/memberships/{}", org, user)).unwrap();
        self.client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn graphql<Q: GraphQLQuery>(&self, vars: Q::Variables) -> Q::ResponseData {
        let response_body = post_graphql::<Q, _>(
            &self.client,
            &self.access_token,
            "https://api.github.com/graphql",
            vars,
        )
        .await
        .unwrap();

        response_body.data.expect("missing response data")
    }
}

#[derive(Deserialize)]
struct UserEmail {
    email: String,
    primary: bool,
}

#[derive(Deserialize)]
pub struct UserAccessToken {
    pub access_token: String,
    // The number of seconds until access_token expires
    pub expires_in: i64,
}

pub struct AppJWT {
    token: String,
}

#[derive(Deserialize)]
pub struct InstallationAccessToken {
    pub token: String,
    pub expires_at: String,
}

impl AppJWT {
    fn header_value(&self) -> HeaderValue {
        let s = format!("Bearer {}", self.token);
        HeaderValue::from_str(&s).unwrap()
    }
}

async fn post_graphql<Q: GraphQLQuery, U: reqwest::IntoUrl>(
    client: &reqwest::Client,
    access_token: &str,
    url: U,
    variables: Q::Variables,
) -> Result<graphql_client::Response<Q::ResponseData>, reqwest::Error> {
    let body = Q::build_query(variables);

    let response = client
        .post(url)
        .header(AUTHORIZATION, &format!("Bearer {}", access_token))
        .json(&body)
        .send()
        .await?;

    response.json().await
}

/// https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-json-web-token-jwt-for-a-github-app
#[derive(Serialize)]
struct JWTClaims {
    /// Issued At. The time that the JWT was created.
    iat: u64,
    /// Expires At. The expiration time of the JWT.
    exp: u64,
    /// Issuer. The ID of your GitHub App.
    iss: String,
    /// Message authentication code algorithm
    alg: String,
}

#[derive(Serialize)]
struct WebhookConfig {
    url: Url,
    content_type: String,
    secret: String,
    insecure_ssl: String,
}
