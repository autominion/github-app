use std::str::FromStr;

use actix_web::http::header::{
    self, Header, HeaderName, HeaderValue, InvalidHeaderValue, TryIntoHeaderValue,
};

#[derive(Debug)]
pub enum XGitHubEvent {
    Ping,
    Installation,
    InstallationRepositories,
    Issues,
    IssueComment,
    Other,
}

impl XGitHubEvent {
    fn as_str(&self) -> &str {
        use XGitHubEvent::*;

        match self {
            Ping => "ping",
            Installation => "installation",
            InstallationRepositories => "installation_repositories",
            Issues => "issues",
            IssueComment => "issue_comment",
            Other => "",
        }
    }
}

impl Header for XGitHubEvent {
    fn name() -> reqwest::header::HeaderName {
        HeaderName::from_static("x-github-event")
    }

    fn parse<M: actix_web::HttpMessage>(msg: &M) -> Result<Self, actix_web::error::ParseError> {
        header::from_one_raw_str(msg.headers().get(Self::name()))
    }
}

impl FromStr for XGitHubEvent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use XGitHubEvent::*;

        match s {
            "ping" => Ok(Ping),
            "installation" => Ok(Installation),
            "installation_repositories" => Ok(InstallationRepositories),
            "issues" => Ok(Issues),
            "issue_comment" => Ok(IssueComment),
            _ => Ok(Other),
        }
    }
}

impl TryIntoHeaderValue for XGitHubEvent {
    type Error = InvalidHeaderValue;

    fn try_into_value(self) -> Result<HeaderValue, Self::Error> {
        HeaderValue::from_str(self.as_str())
    }
}
