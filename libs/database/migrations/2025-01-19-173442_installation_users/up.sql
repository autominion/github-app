create type user_role as enum (
    'admin',
    'member'
);

create table installation_users (
    installation_id uuid not null references installations (
        id
    ) on delete cascade,
    user_id uuid not null references users (id) on delete cascade,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    role user_role not null default 'member',
    primary key (installation_id, user_id)
);

select diesel_manage_updated_at('installation_users');
