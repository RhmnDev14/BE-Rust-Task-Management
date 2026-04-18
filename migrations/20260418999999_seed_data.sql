-- Aktifkan ekstensi UUID jika belum
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Hapus data lama (opsional)
TRUNCATE TABLE tasks CASCADE;
TRUNCATE TABLE users CASCADE;

-- Insert Dummy Users
-- Password untuk semua user adalah 'password123' 
-- Hash ini menggunakan Argon2id yang diverifikasi
INSERT INTO users (id, username, email, password_hash, avatar_url) VALUES
('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'budi_santoso', 'budi@example.com', '$argon2id$v=19$m=19456,t=2,p=1$588VYSATv7ihAOscgdRMYg$nmYItNsPqCOslARDRPEFUAccnMp+v9qL7rkkPD0Du4g', 'https://ui-avatars.com/api/?name=Budi+Santoso'),
('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'siti_aminah', 'siti@example.com', '$argon2id$v=19$m=19456,t=2,p=1$588VYSATv7ihAOscgdRMYg$nmYItNsPqCOslARDRPEFUAccnMp+v9qL7rkkPD0Du4g', 'https://ui-avatars.com/api/?name=Siti+Aminah');

-- Insert Dummy Tasks
INSERT INTO tasks (id, task_name, description, id_user) VALUES
(uuid_generate_v4(), 'Belajar Rust Dasar', 'Mempelajari ownership, borrowing, dan lifetimes di Rust.', 'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'),
(uuid_generate_v4(), 'Implementasi Axum', 'Membuat REST API sederhana menggunakan framework Axum.', 'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'),
(uuid_generate_v4(), 'Integrasi PostgreSQL', 'Menghubungkan aplikasi Rust dengan database PostgreSQL menggunakan SQLx.', 'b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22'),
(uuid_generate_v4(), 'Menulis Unit Test', 'Memastikan kode berjalan dengan benar melalui pengujian otomatis.', 'b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22');
