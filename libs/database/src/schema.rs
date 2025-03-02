// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "task_failure_reason"))]
    pub struct TaskFailureReason;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "task_status"))]
    pub struct TaskStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_role"))]
    pub struct UserRole;
}

diesel::table! {
    agent_configs (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        container_registry_host -> Text,
        container_registry_username -> Nullable<Text>,
        container_registry_password -> Nullable<Text>,
        container_image -> Text,
    }
}

diesel::table! {
    installation_repository_users (installation_id, repository_id, user_id) {
        installation_id -> Uuid,
        repository_id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UserRole;

    installation_users (installation_id, user_id) {
        installation_id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        role -> UserRole,
    }
}

diesel::table! {
    installations (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        github_id -> Int8,
        created_by_github_id -> Nullable<Text>,
        suspended_at -> Nullable<Timestamptz>,
        suspended_by_github_id -> Nullable<Text>,
    }
}

diesel::table! {
    installations_repositories (installation_id, repository_id) {
        installation_id -> Uuid,
        repository_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        active -> Bool,
    }
}

diesel::table! {
    llm_interactions (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        task_id -> Uuid,
        request -> Nullable<Jsonb>,
        response -> Nullable<Jsonb>,
    }
}

diesel::table! {
    repositories (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        github_id -> Text,
        github_full_name -> Text,
        github_private -> Bool,
        default_agent_config_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    task_compute_usage (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        task_id -> Uuid,
        compute_usage_start_timestamp -> Timestamptz,
        compute_usage_end_timestamp -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TaskStatus;
    use super::sql_types::TaskFailureReason;

    tasks (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by_id -> Uuid,
        installation_id -> Nullable<Uuid>,
        repository_id -> Uuid,
        github_issue_id -> Text,
        github_issue_number -> Int8,
        status -> TaskStatus,
        completion_description -> Nullable<Text>,
        failure_description -> Nullable<Text>,
        failure_reason -> Nullable<TaskFailureReason>,
        agent_config_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        active -> Bool,
        joined_waitlist_at -> Nullable<Timestamptz>,
        github_id -> Text,
        github_email -> Nullable<Text>,
        github_name -> Nullable<Text>,
        github_login -> Text,
        github_access_token -> Nullable<Text>,
        github_access_token_expires_at -> Nullable<Timestamptz>,
        openrouter_key -> Nullable<Text>,
        openrouter_code_verifier -> Nullable<Text>,
    }
}

diesel::joinable!(installation_repository_users -> installations (installation_id));
diesel::joinable!(installation_repository_users -> users (user_id));
diesel::joinable!(installation_users -> installations (installation_id));
diesel::joinable!(installation_users -> users (user_id));
diesel::joinable!(installations_repositories -> installations (installation_id));
diesel::joinable!(installations_repositories -> repositories (repository_id));
diesel::joinable!(llm_interactions -> tasks (task_id));
diesel::joinable!(repositories -> agent_configs (default_agent_config_id));
diesel::joinable!(task_compute_usage -> tasks (task_id));
diesel::joinable!(tasks -> agent_configs (agent_config_id));
diesel::joinable!(tasks -> installations (installation_id));
diesel::joinable!(tasks -> repositories (repository_id));
diesel::joinable!(tasks -> users (created_by_id));

diesel::allow_tables_to_appear_in_same_query!(
    agent_configs,
    installation_repository_users,
    installation_users,
    installations,
    installations_repositories,
    llm_interactions,
    repositories,
    task_compute_usage,
    tasks,
    users,
);
