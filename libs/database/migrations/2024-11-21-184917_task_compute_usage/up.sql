create table task_compute_usage (
    id uuid primary key default uuidv7(),
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    task_id uuid not null references tasks (id),
    compute_usage_start_timestamp timestamptz not null,
    compute_usage_end_timestamp timestamptz
);

select diesel_manage_updated_at('task_compute_usage');
