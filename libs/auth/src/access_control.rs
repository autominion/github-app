use uuid::Uuid;

use database::{Database, InstallationRepository, User};

pub struct CreateTask {
    pub user: User,
    pub inst_repo: InstallationRepository,
}

/// Check that the user can create a task for the given repository.
pub async fn github_user_can_create_task(
    db: &Database,
    github_user_id: &str,
    github_repo_id: &str,
) -> Option<CreateTask> {
    let mut conn = db.conn().await;

    // User active: the user must be active
    let user = conn.get_user_by_github_id(github_user_id).await.unwrap();
    if !user.active {
        return None;
    }

    // Membership: the user must have repository membership via either:
    //   - `installation_users`
    //   - `installation_repository_users`
    let inst_repo = conn
        .installation_repository_for_authorized_github_user(github_user_id, github_repo_id)
        .await?;

    // Repository active: the repository must be active
    if !inst_repo.active {
        return None;
    }

    Some(CreateTask { user, inst_repo })
}

/// Check that the user is active.
pub async fn user_is_active(db: &Database, user_id: Uuid) -> bool {
    let mut conn = db.conn().await;

    // User active: the user must be active
    let user = conn.get_user(&user_id).await;
    user.active
}

/// Check that the user can view the task.
pub async fn user_can_read_task(db: &Database, user_id: Uuid, task_id: Uuid) -> bool {
    let mut conn = db.conn().await;

    // User active: the user must be active
    let user = conn.get_user(&user_id).await;
    if !user.active {
        return false;
    }

    let task = conn.get_task(&task_id).await;

    // Membership: the user must have repository membership via either:
    //   - `installation_users`
    //   - `installation_repository_users`
    // for the repository associated with the task
    if !conn.user_is_member_of_repository(user_id, task.repository_id).await {
        return false;
    }

    true
}

/// Check that the user can administer the repository.
pub async fn user_can_admin_repo(db: &Database, user_id: Uuid, repo_id: Uuid) -> bool {
    let mut conn = db.conn().await;

    // User active: the user must be active
    let user = conn.get_user(&user_id).await;
    if !user.active {
        return false;
    }

    // Membership: the user must have `Admin` membership via `installation_users`
    if !conn.user_is_admin_of_repository(user_id, repo_id).await {
        return false;
    }

    true
}
