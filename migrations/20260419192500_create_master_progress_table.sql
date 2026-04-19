-- Add migration script here
CREATE TABLE IF NOT EXISTS master_progress (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL
);
