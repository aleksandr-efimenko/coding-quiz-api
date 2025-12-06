# Coding Quiz API

A RESTful API for managing coding quizzes, built with Rust, Actix-web, PostgreSQL, and SQLx.

## Features

-   **User Authentication**: JWT-based registration and login.
-   **Categories**: Organize quizzes into categories.
-   **Quizzes**: Create, retrieve, and list quizzes. Support for multiple-choice questions.
-   **Problem Solving**: Submit answers and get immediate feedback.
-   **Data Persistence**: All data stored in PostgreSQL.
-   **OpenAPI Documentation**: Interactive API docs via Swagger UI.

## Prerequisites

-   **Rust** (latest stable)
-   **PostgreSQL** (running locally or via Docker)
-   **SQLx CLI** (for migrations): `cargo install sqlx-cli`

## Setup

1.  **Clone the repository**.

2.  **Environment Variables**:
    Create a `.env` file in the root directory:
    ```env
    DATABASE_URL=postgres://user:password@localhost:5432/coding_quiz_api
    RUST_LOG=info
    ```
    *Replace `user`, `password`, and database name with your postgres credentials.*

3.  **Database Setup**:
    ```bash
    # Create database
    sqlx database create

    # Run migrations (creates tables: quizzes, questions, users, categories)
    sqlx migrate run
    ```
    *Note: A helper migration script is also available in `src/bin/migrate.rs` if you cannot install sqlx-cli.*

4.  **Run the Server**:
    ```bash
    cargo run
    ```
    The server will start at `http://127.0.0.1:8080`.

5.  **View Documentation**:
    Open [http://127.0.0.1:8080/swagger-ui/](http://127.0.0.1:8080/swagger-ui/) in your browser.
    
    > **Note**: To test protected endpoints in Swagger UI:
    > 1. Click the **Authorize** button (top right).
    > 2. Enter the JWT token (Bearer <token>) or just the token string depending on configuration.
    > 3. Click **Authorize** and then **Close**.

## API Reference

### Authentication

#### Register (`POST /auth/register`)
Create a new user.
```json
{
  "username": "testuser",
  "password": "securepassword"
}
```

#### Login (`POST /auth/login`)
Authenticate and receive a JWT token.
```json
{
  "username": "testuser",
  "password": "securepassword"
}
```
**Response**:
```json
{
  "token": "eyJhbGciOiJIUzI1Ni..."
}
```

### Categories

#### List Categories (`GET /categories`)
Get all categories.

#### Create Category (`POST /categories`)
**Requires Auth (Bearer Token)**
```json
{
  "name": "Rust Programming"
}
```

### Quizzes

#### List Quizzes (`GET /quizzes`)
Get all quizzes.
-   **Filter by Category**: `GET /quizzes?category_id=<uuid>`

#### Create Quiz (`POST /quizzes`)
**Requires Auth (Bearer Token)**
```json
{
  "title": "Basic Types",
  "category_id": "<category_uuid>",
  "questions": [
    {
      "text": "What is the size of i32?",
      "options": [
        { "text": "32 bits", "is_correct": true },
        { "text": "64 bits", "is_correct": false }
      ]
    }
  ]
}
```

#### Get Quiz (`GET /quizzes/{id}`)
Get a quiz by ID (questions included, correct answers hidden).

#### Solve Question (`POST /quizzes/{id}/solve`)
Submit an answer.
```json
{
  "question_id": "<uuid>",
  "option_id": "<uuid>"
}
```
**Response**:
```json
{
  "correct": true,
  "message": "Correct!"
}
```

## Testing

You can use `curl` to test endpoints.

**Example: Get Health**
```bash
curl http://localhost:8080/health
```
