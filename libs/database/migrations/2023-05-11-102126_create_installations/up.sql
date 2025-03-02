create table installations (
    id uuid primary key default uuidv7(),
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    github_id bigint not null unique,
    created_by_github_id text,
    suspended_at timestamptz,
    suspended_by_github_id text
);

select diesel_manage_updated_at('installations');
