create table agent_configs (
    id uuid primary key default uuidv7(),
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    container_registry_host text not null,
    container_registry_username text,
    container_registry_password text,
    container_image text not null
);

select diesel_manage_updated_at('agent_configs');

alter table tasks
add column agent_config_id uuid references agent_configs (
    id
) on delete set null;

alter table repositories
add column default_agent_config_id uuid references agent_configs (
    id
) on delete set null;
