-- Add migration script here
CREATE TABLE IF NOT EXISTS user_role (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES master_role(id) ON DELETE CASCADE,
    UNIQUE(user_id, role_id)
);
