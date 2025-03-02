use git2::build::RepoBuilder;
use tempfile::TempDir;
use tokio::task::spawn_blocking;

pub async fn push_task_branch(repo_url: &str, ref_name: &str) {
    let repo_url = repo_url.to_owned();
    let ref_name = ref_name.to_owned();
    spawn_blocking(move || {
        push_task_branch_blocking(&repo_url, &ref_name);
    })
    .await
    .unwrap();
}

fn push_task_branch_blocking(repo_url: &str, ref_name: &str) {
    let temp_dir = TempDir::new().unwrap();
    let mut repo_builder = RepoBuilder::new();
    repo_builder.bare(true);
    let repo = repo_builder.clone(repo_url, temp_dir.path()).unwrap();
    let mut remote = repo.remote("target", repo_url).unwrap();
    let refspecs = [format!("refs/heads/main:{}", ref_name)];
    remote.push(&refspecs, None).unwrap();
}
