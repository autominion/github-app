create table users (
    id uuid primary key default uuidv7(),
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    active boolean not null default false,
    joined_waitlist_at timestamptz,
    github_id text not null unique,
    github_email text,
    github_name text,
    github_login text not null,
    github_access_token text,
    github_access_token_expires_at timestamptz
);

select diesel_manage_updated_at('users');
