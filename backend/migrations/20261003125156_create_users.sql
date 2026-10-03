CREATE TABLE users (
    id VARCHAR(26) PRIMARY KEY,

    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    deleted_at TIMESTAMPTZ NULL,
    deleted_by VARCHAR(26) NULL
);

CREATE INDEX idx_users_deleted_at
    ON users(deleted_at);