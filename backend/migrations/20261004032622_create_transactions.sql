
Create Table transactions (
      id VARCHAR(26) PRIMARY KEY,
    category_id varchar(26) not null references categories(id),
    type varchar(50) not null,
    amount numeric not null,
    description text not null,
    date date not null,
    user_id varchar(26) not null references users(id),


    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    deleted_at TIMESTAMPTZ NULL,
    deleted_by VARCHAR(26) NULL
);

create index idx_transactions_deleted_at
    on transactions(deleted_at);