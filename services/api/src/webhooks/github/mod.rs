use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

mod events;
mod headers;
mod verify_signature;

use events::*;
use headers::XGitHubEvent;
use verify_signature::verify_github_signature;

use config::Config;
use database::Database;
use github::GitHub;

#[post("/webhooks/github")]
pub async fn github_hook(
    event_header: web::Header<XGitHubEvent>,
    body: web::Bytes,
    req: HttpRequest,
    config: web::Data<Config>,
    db: web::Data<Database>,
    github: web::Data<GitHub>,
) -> impl Responder {
    let signature_header = match req.headers().get("X-Hub-Signature-256") {
        Some(sig) => sig.to_str().unwrap_or_default(),
        None => {
            log::warn!("Missing X-Hub-Signature-256 header");
            return HttpResponse::BadRequest().finish();
        }
    };

    if !verify_github_signature(&config.github_app_webhook_secret, &body, signature_header) {
        log::warn!("Invalid signature");
        return HttpResponse::Unauthorized().finish();
    }

    println!("GitHub hook fired {event_header:?}");

    use XGitHubEvent::*;
    let db = db.get_ref().clone();
    let github = github.get_ref().clone();
    let config = config.get_ref();

    match event_header.into_inner() {
        Ping => PingEvent::handle_bytes(&body, config, db, github).await.unwrap(),
        Installation => InstallationEvent::handle_bytes(&body, config, db, github).await.unwrap(),
        InstallationRepositories => {
            InstallationRepositoriesEvent::handle_bytes(&body, config, db, github).await.unwrap()
        }
        Issues => IssuesEvent::handle_bytes(&body, config, db, github).await.unwrap(),
        IssueComment => IssueCommentEvent::handle_bytes(&body, config, db, github).await.unwrap(),
        Other => (),
    }

    HttpResponse::Ok().finish()
}
