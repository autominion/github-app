create type task_status as enum (
    'queued',
    'running',
    'completed',
    'failed'
);

create table tasks (
    id uuid primary key default uuidv7(),
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    created_by_id uuid references users (id) on delete cascade not null,
    -- Keep the task if the installation is deleted
    installation_id uuid references installations (id) on delete set null,
    -- Remove the task if the repository is deleted
    repository_id uuid references repositories (id) on delete cascade not null,
    github_issue_id text not null,
    github_issue_number bigint not null,
    status task_status not null
);

select diesel_manage_updated_at('tasks');
