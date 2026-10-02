-- Add migration script here
-- users table: one row per registered account

CREATE TABLE users(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- todos table: one row per todo item, owned by exactly one user
CREATE TABLE todos(
    id  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);


-- Index for the most common query: "give me this user's todos"
CREATE INDEX idx_todo_user_id ON todos(user_id);