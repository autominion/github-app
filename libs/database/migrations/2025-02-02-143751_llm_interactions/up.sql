create table llm_interactions (
    id uuid primary key default uuidv7(),
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    task_id uuid references tasks (id) on delete cascade not null,
    request jsonb,
    response jsonb
);
