use std::time::Duration;

use config::Config;
use config::DispatchMode;
use database::Database;
use github::GitHub;
use object_storage::S3;
use uuid::Uuid;

use crate::vm::CommandResult;
use crate::vm::{AwsVm, LocalVM, VirtualMachine};

use super::git;
use super::Args;

const TOKEN_LIFETIME: Duration = Duration::from_secs(60 * 60);

pub struct Job {
    pub issue_id: String,
    pub repo_github_id: String,
    pub repo_name: String,
    pub task_id: Uuid,
}

pub async fn run(
    config: &Config,
    args: &Args,
    db: Database,
    github: &GitHub,
    s3: &S3,
    token_signer: &auth::TokenSigner,
    job: &Job,
) {
    let jwt = github.github_app_jwt();
    let access_token = github.installation_access_token(&jwt).await;
    let github_inst = github.with_access(&access_token.token);

    let issue_info = github_inst.issue_info(&job.issue_id).await;
    let description = issue_info.body;
    println!("{}", description);

    let task_url = config.web_base_url.join(&format!("/tasks/{}", job.task_id)).unwrap();
    let body = format!("Started working on the [task]({task_url}).");
    github_inst.add_comment(&job.issue_id, &body).await;

    let repo_numeric_id = github_inst.repo_numeric_id_by_node_id(&job.repo_github_id).await;

    let repo_access_token =
        github.create_scoped_access_token(&github.github_app_jwt(), repo_numeric_id).await;

    let repo_url =
        format!("https://oauth2:{}@github.com/{}", &repo_access_token.token, job.repo_name);

    // For now we assume that the task id is the branch name
    let branch_ref_name = format!("refs/heads/{}", job.task_id);
    git::push_task_branch(&repo_url, &branch_ref_name).await;

    let agent_token = auth::issue_agent_token(token_signer, &job.task_id, TOKEN_LIFETIME);

    if matches!(config.dispatch_mode, DispatchMode::None) {
        return;
    }

    let usage_start = chrono::Utc::now();

    let usage = {
        let mut conn = db.conn().await;
        conn.start_compute_usage(job.task_id, usage_start).await.unwrap()
    };

    let log_output = if args.local {
        run_vm::<LocalVM>(config, &agent_token).await
    } else {
        run_vm::<AwsVm>(config, &agent_token).await
    };

    println!("{}", log_output);

    let usage_end = chrono::Utc::now();

    {
        let mut conn = db.conn().await;
        conn.end_compute_usage(usage.id, usage_end).await.unwrap();
    }

    let pr_title = format!("Pull request from {}", config.service_name);
    let pr_body = r#"---

AI-generated. Review carefully.
"#;

    github_inst
        .create_pull_request(&job.repo_github_id, &pr_title, pr_body, &branch_ref_name)
        .await;

    let body = format!("[Task]({task_url}) completed.");
    github_inst.add_comment(&job.issue_id, &body).await;
    s3.upload_log_for_task(&job.task_id, log_output).await.unwrap();
}

async fn run_vm<V: VirtualMachine>(config: &Config, access_token: &str) -> String {
    let mut vm = V::create(config).await;

    // Setups the VM with the necessary tools.
    vm.install_docker().await;

    let container_registry_host = &config.default_agent_container_registry_host;
    let container_registry_username = &config.default_agent_container_registry_username;
    let container_registry_password = &config.default_agent_container_registry_password;
    let container_image = &config.default_agent_container_image;

    // Login to the container registry and pull the image.
    vm.run_command(&format!(
        "docker login -u {} -p {} {}",
        shlex::try_quote(container_registry_username).unwrap(),
        shlex::try_quote(container_registry_password).unwrap(),
        shlex::try_quote(container_registry_host).unwrap()
    ))
    .await;

    let formatted_image = format!("{}/{}", container_registry_host, container_image);
    let registry_and_image = shlex::try_quote(&formatted_image).unwrap();

    vm.run_command(&format!("docker pull {}", registry_and_image)).await;

    let registry_host = shlex::try_quote(container_registry_host).unwrap();

    // Logout from the container registry before running the container.
    vm.run_command(&format!("docker logout {}", registry_host)).await;

    let api_base_url = config.web_base_url.join("/api/").unwrap();

    // Run the agent software in detached mode.
    let CommandResult { log_output, .. } = vm.run_command(&format!(
        "docker run --runtime=sysbox-runc --pull never -e MINION_API_BASE_URL={} -e MINION_API_TOKEN={} {}",
        api_base_url, access_token, registry_and_image
    ))
    .await;

    // Disconnect the SSH connection.
    vm.detach().await;

    vm.destroy().await;

    log_output
}
