# Coding Quiz API (In-Memory Learning Platform)

A RESTful API designed for developers to create quizzes and practice coding concepts. Completely in-memory, stateless, and public. Built with Rust and Actix-web.

## Architecture: In-Memory & Public

This API operates as a stateless service:
1.  **Public Access**: No authentication required. All endpoints are open.
2.  **In-Memory**: All data (quizzes, categories) is loaded from JSON seed files at startup.
3.  **Stateless**: No database persistence. Restarting the server resets the state.

## Features

-   **Public API**: No API Keys or JWTs needed.
-   **In-Memory Speed**: Extremely fast response times.
-   **JSON Seeding**: Easy to extend content by adding JSON files to `seed/`.
-   **Smart Content Delivery**: `GET /quizzes/random` allows fetching random quizzes, filtering by tag.
-   **OpenAPI Documentation**: Interactive API docs via Swagger UI.

## Prerequisites

-   **Rust** (latest stable)

## Setup

1.  **Clone the repository**.

2.  **Run the Server**:
    ```bash
    cargo run
    ```

## API Documentation

The API documentation is available at:

*   **Swagger UI**: `http://localhost:8080/swagger-ui/`
    *   Endpoints: Quizzes, Categories, Solving.

### Key Features
*   **Search**: Filter quizzes by title or category.
*   **Random Quiz**: Get a random quiz to solve.
*   **Tags**: Filter content by specific topics (e.g., `rust`, `javascript`).

## Data Seeding

The application automatically loads quizzes from the `seed/` directory on startup.
-   To add more quizzes, simply add a valid JSON file to `seed/javascript/` (or create new folders) and restart the server.

## API Reference

### 1. Management (Public)
-   `POST /categories`: Create a new category (Ephemeral).
-   `POST /quizzes`: Create a new quiz (Ephemeral).
-   `PUT /quizzes/{id}`: Update a quiz.
-   `DELETE /quizzes/{id}`: Delete a quiz.

### 2. Consumption (Public)
-   `GET /categories`: List all categories.
-   `GET /quizzes`: List all quizzes.
-   `GET /quizzes/{id}`: Get details for a specific quiz.
-   `GET /quizzes/random`: Get a random quiz.
    -   `?tag=rust` (Optional): Filter by tag.

#### Solving
-   `POST /quizzes/{id}/solve`: Submit an answer.
    ```json
    {
      "question_id": "...",
      "option_id": "..."
    }
    ```
    *Returns correct/incorrect status and explanation.*

## Testing REST Clients
Use the `.rest` files in the `rest_client/` directory with the [REST Client](https://marketplace.visualstudio.com/items?itemName=humao.rest-client) extension for VS Code.

-   `rest_client/management.rest`: For creating content.
-   `rest_client/consumption.rest`: For solving quizzes.

## License
MIT
