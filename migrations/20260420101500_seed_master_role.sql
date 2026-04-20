-- Add migration script here
INSERT INTO master_role (id, name) VALUES 
(uuid_generate_v4(), 'Admin'),
(uuid_generate_v4(), 'Manager'),
(uuid_generate_v4(), 'User')
ON CONFLICT DO NOTHING;
