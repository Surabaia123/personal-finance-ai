
Create Table categories (
    id VARCHAR(26) PRIMARY KEY,
    name varchar(255) not null,
    user_id varchar(26) not null references users(id),
type varchar(50) not null,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    deleted_at TIMESTAMPTZ NULL,
    deleted_by VARCHAR(26) NULL
);

create index idx_categories_deleted_at
    on categories(deleted_at);
