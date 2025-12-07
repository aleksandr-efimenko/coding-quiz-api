# Coding Quiz API (B2B Managed Learning Platform)

A RESTful API designed for a B2B2C model where developers can create quizzes and manage their own users' learning progress. Built with Rust, Actix-web, PostgreSQL, and SQLx.

## Architecture: Managed B2B

This API operates on a B2B2C model:
1.  **Developers** register and manage content (Quizzes, Categories) and obtain an API Key.
2.  **Developers** register their **Users** (End-Users) via the API throughout their own application lifecycle.
3.  **Consumption**: Developers use their API Key to fetch quizzes, submit answers on behalf of their users, and track user history.

## Features

-   **Dual Authentication**:
    -   **JWT (Bearer)**: For Management (Developers creating quizzes, generating API keys).
    -   **API Key (`X-API-Key`)**: For Consumption (Client apps registering users, solving quizzes).
-   **User Management**: Developers can create and delete "Quiz Users" scoped to their account.
-   **Stateful Learning**: Tracks which quizzes a user has solved.
-   **Smart Content Delivery**: `GET /quizzes/random` allows fetching random unsolved quizzes, filtering by tag or category.
-   **Verified Content**: Questions must be explicitly verified (`PUT /questions/{id}/verify`) to be visible to consumers.
-   **OpenAPI Documentation**: Interactive API docs via Swagger UI.

## Prerequisites

-   **Rust** (latest stable)
-   **PostgreSQL** (running locally or via Docker)
-   **SQLx CLI** (for migrations): `cargo install sqlx-cli`
-   **Docker** (optional, for containerized run)

## Setup

1.  **Clone the repository**.

2.  **Environment Variables**:
    Create a `.env` file in the root directory:
    ```env
    DATABASE_URL=postgres://postgres:password@localhost:5432/coding_quiz_api
    RUST_LOG=info
    ```

3.  **Database Setup**:
    ```bash
    # Create database
    sqlx database create

    # Run migrations
    sqlx migrate run
    ```

4.  **Run the Server**:
    **Local:**
    ```bash
    # Ensure DB is running (e.g., via docker compose up db)
    cargo run
    ```
    *Note: If building gives "Connection refused" errors, run `SQLX_OFFLINE=true cargo build`.*

    **Docker:**
    ```bash
    # Build and run
    docker compose up --build
    ```

## API Documentation

The API documentation is split into two interfaces:

*   **Public (Consumption)**: `http://localhost:8080/swagger-ui/public/`
    *   Auth: `X-API-Key`
    *   Endpoints: Quizzes, User History, Stats, Tags.
*   **Private (Management)**: `http://localhost:8080/swagger-ui/private/`
    *   Auth: `Bearer <JWT>` (API), **Basic Auth** (UI).
    *   **UI Credentials**: `admin` / `password` (Default).
    *   Set `SWAGGER_USERNAME` and `SWAGGER_PASSWORD` to change.
    *   Endpoints: Content Creation, API Key Generation.

### Key Features
*   **Search**: Filter quizzes by title (`GET /quizzes?search=Rust`).
*   **Difficulty**: Quizzes are classified as `Easy`, `Medium`, or `Hard`.
*   **Advanced Filtering**: Filter random quizzes by multiple tags and difficulty.
*   **User Stats**: Track user accuracy and total quizzes taken.

## Database Management

### Migrations
The project uses `sqlx` for migrations.
-   **Run Migrations**: `sqlx migrate run`
-   **Create Migration**: `sqlx migrate add <name>`
-   **Revert Last**: `sqlx migrate revert`

### Scripts & Seeding
We included utility scripts in the `scripts/` and `seed/` directories.

**Seeding the Database:**
To populate the database with initial categories and quizzes:
```bash
# Verify database connection first
cargo run --bin check_db

# Run the seeder
cargo run --bin seed
```
*Note: Make sure your server or DB is reachable.*

**Resetting the Database (Docker):**
If you encounter `VersionMissing` or sync errors:
```bash
docker compose down -v  # Deletes volume!
docker compose up --build
```

## API Reference

### 1. Management (Developer - JWT)
*Authentication*: `Authorization: Bearer <jwt_token>`

-   `POST /developer/register`: Register as a developer.
-   `POST /developer/login`: Login to get JWT.
-   `POST /developer/api-keys`: Generate an API Key for your application.
-   `POST /quizzes`: Create a new quiz.
-   `PUT /questions/{id}/verify`: Verify a question (make it public).
-   `PUT /quizzes/{id}`: Update a quiz.
-   `DELETE /quizzes/{id}`: Delete a quiz.

### 2. Consumption (Client App - API Key)
*Authentication*: `X-API-Key: <your_api_key>`

#### User Management
-   `POST /users`: Register a new end-user (e.g., `alice@example.com`).
-   `DELETE /users/{email}`: Delete a user and their history.
-   `GET /users/{email}/history`: Get all quiz attempts for a user.

#### Quiz Consumption
-   `GET /quizzes`: List all quizzes.
-   `GET /quizzes/{id}`: Get details for a specific quiz (Verified questions only).
-   `GET /quizzes/random`: Get a random quiz.
    -   `?user_email=alice@example.com` (Required): Scopes to user history.
    -   `?tag=rust` (Optional): Filter by tag.
    -   `?category_id=...` (Optional): Filter by category.
    -   `?include_solved=true` (Optional): Include quizzes already solved (default false).

#### Solving
-   `POST /quizzes/{id}/solve`: Submit an answer.
    ```json
    {
      "user_email": "alice@example.com",
      "question_id": "...",
      "option_id": "..."
    }
    ```
    *Records the attempt in the user's history.*

## Testing REST Clients
Use the `.rest` files in the `requests/` directory with the [REST Client](https://marketplace.visualstudio.com/items?itemName=humao.rest-client) extension for VS Code.

-   `requests/management.rest`: For registering, login, and creating content.
-   `requests/consumption.rest`: For simulating the client app flow (register user -> solve -> history).

## License
MIT
