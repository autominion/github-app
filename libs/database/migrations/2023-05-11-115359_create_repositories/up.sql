create table repositories (
    id uuid primary key default uuidv7(),
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    github_id text not null unique,
    github_full_name text not null,
    github_private boolean not null
);

select diesel_manage_updated_at('repositories');

create table installations_repositories (
    installation_id uuid references installations (id) on delete cascade,
    repository_id uuid references repositories (id) on delete cascade,
    primary key (installation_id, repository_id),
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    active boolean not null default false
);

select diesel_manage_updated_at('installations_repositories');
