CREATE TABLE installation_repository_users (
    installation_id uuid NOT NULL REFERENCES installations (
        id
    ) ON DELETE CASCADE,
    repository_id uuid NOT NULL,
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (installation_id, repository_id, user_id),
    FOREIGN KEY (
        installation_id, repository_id
    ) REFERENCES installations_repositories (
        installation_id, repository_id
    ) ON DELETE CASCADE
);

SELECT diesel_manage_updated_at('installation_repository_users');
