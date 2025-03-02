use std::sync::Arc;
use std::time::Duration;

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_jwt_auth_middleware::{use_jwt::UseJWTOnApp, Authority, TokenSigner};
use actix_web::{middleware, web, App, HttpServer};
use jwt_compact::alg::{Ed25519, SigningKey, VerifyingKey};

use config::Config;
use database::Database;

mod api;
mod git_proxy_auth;
mod probes;
mod webhooks;

use auth::SessionId;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load configuration
    let config = Config::load();

    // Connect to database
    let db = Database::connect_and_init(config.postgres_url.as_str()).await;

    // Create API clients
    let github = github::GitHub::new(config.clone());
    let s3 = object_storage::S3::new(&config).expect("Failed to create S3 client");

    // Keys for session handling
    let jwt_private_key = config.jwt_expanded_private_key.clone();
    let secret_key = move || SigningKey::from_slice(&jwt_private_key).unwrap();
    let public_key = VerifyingKey::from_slice(&config.jwt_public_key).unwrap();

    // Simple rate limiter
    let governor_conf = GovernorConfigBuilder::default().finish().unwrap();

    let (host, port) = (config.host.clone(), config.port);

    HttpServer::new(move || {
        let authority = Authority::<SessionId, Ed25519, _, _>::new()
            .enable_header_tokens(true)
            .enable_authorization_header(true)
            .refresh_authorizer(|| async move { Ok(()) })
            .token_signer(Some(
                TokenSigner::new()
                    .signing_key(secret_key())
                    .algorithm(Ed25519)
                    .access_token_lifetime(Duration::from_secs(60 * 60))
                    .refresh_token_lifetime(Duration::from_secs(12 * 60 * 60))
                    .build()
                    .expect("Failed to create TokenSigner"),
            ))
            .verifying_key(public_key)
            .build()
            .expect("Failed to create authority");

        let git_auth_validator = {
            let db = Arc::new(db.clone());
            let github = Arc::new(github.clone());
            let public_key = Arc::new(public_key);
            move |req, credentials| {
                git_proxy_auth::basic_auth_validator(
                    db.clone(),
                    github.clone(),
                    public_key.clone(),
                    req,
                    credentials,
                )
            }
        };

        App::new()
            .app_data(web::Data::new(github.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(s3.clone()))
            .service(api::auth::github::auth)
            .service(api::auth::github::auth_code)
            .service(webhooks::github::github_hook)
            .service(probes::readiness)
            .service(probes::healthz)
            .service(api::auth::openrouter::openrouter_auth_code)
            .service(git_proxy::scope("/api/agent/git", git_auth_validator))
            .service(actix_files::Files::new("/static", &config.static_dir))
            .use_jwt(
                authority,
                web::scope("").service(api::auth::openrouter::openrouter_connect).service(
                    web::scope("/api")
                        .service(api::user::user_info)
                        .service(api::user::waitlist_join)
                        .service(api::user::waitlist_leave)
                        .service(api::user::openrouter_status)
                        .service(api::user::disconnect_openrouter)
                        .service(api::repos::list_repos)
                        .service(api::repos::get_repo)
                        .service(api::repos::repos_activate)
                        .service(api::repos::repos_deactivate)
                        .service(api::repos::list_repo_users)
                        .service(api::repos::add_repo_user)
                        .service(api::repos::delete_repo_user)
                        .service(api::tasks::list_tasks)
                        .service(api::tasks::task_details)
                        .service(api::tasks::task_logs)
                        .service(api::tasks::task_poll)
                        .service(api::agent::scope())
                        .service(api::chat::scope()),
                ),
            )
            .wrap(middleware::NormalizePath::new(middleware::TrailingSlash::Trim))
            .wrap(middleware::Logger::default())
            .wrap(Governor::new(&governor_conf))
    })
    .bind((host, port))?
    .run()
    .await
}
