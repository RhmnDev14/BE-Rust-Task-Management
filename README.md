# Rust Task Management API

REST API for user management built with Rust, Axum, SQLx, and clean architecture.

## Tech Stack

- **Framework**: Axum
- **Database**: PostgreSQL (SQLx)
- **Runtime**: Tokio
- **Authentication**: JWT (Argon2 for password hashing)
- **Documentation**: Swagger UI (Utoipa)

## Prerequisites

- Rust (latest stable)
- PostgreSQL
- `sqlx-cli` (for migrations)

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

## Setup

1.  **Clone the repository**

2.  **Environment Configuration**
    Create a `.env` file in the root directory:

    ```env
    DATABASE_URL=postgres://postgres:password@localhost:5432/task_management
    JWT_SECRET=supersecretkey
    HOST=127.0.0.1
    PORT=3000
    ```

3.  **Database Setup**

    You can run PostgreSQL using Podman (or Docker):

    ```bash
    podman run --name postgres -e POSTGRES_PASSWORD=password -p 5432:5432 -d postgres
    ```

    Then create the database and run migrations:

    ```bash
    sqlx database create
    sqlx migrate run
    ```

## Running the Application

```bash
cargo run
```

The server will start at `http://127.0.0.1:3000`.

## API Documentation

Swagger UI is available at:

[http://localhost:3000/swagger-ui](http://localhost:3000/swagger-ui)

## Project Structure

- `src/domain`: Entities and Repository interfaces.
- `src/infrastructure`: Database implementations and external services.
- `src/application`: Business logic and use cases.
- `src/api`: Axum handlers and router configuration.
