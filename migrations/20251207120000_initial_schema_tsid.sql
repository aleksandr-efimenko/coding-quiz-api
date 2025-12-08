-- Consolidated Migration for TSID Refactor

-- 1. Developers (formerly Users)
CREATE TABLE developers (
    id BIGINT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL
);

-- 2. API Keys
CREATE TABLE api_keys (
    id BIGINT PRIMARY KEY,
    developer_id BIGINT NOT NULL REFERENCES developers(id),
    key_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. Usage Logs
CREATE TABLE usage_logs (
    id BIGINT PRIMARY KEY,
    api_key_id BIGINT NOT NULL REFERENCES api_keys(id),
    endpoint VARCHAR(255) NOT NULL,
    status_code INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 4. Categories
CREATE TABLE categories (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

-- 5. Quizzes
CREATE TABLE quizzes (
    id BIGINT PRIMARY KEY,
    title TEXT NOT NULL,
    category_id BIGINT REFERENCES categories(id)
);

-- 6. Questions
CREATE TABLE questions (
    id BIGINT PRIMARY KEY,
    quiz_id BIGINT NOT NULL REFERENCES quizzes(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    explanation TEXT,
    UNIQUE(text)
);

-- 7. Question Options
CREATE TABLE question_options (
    id BIGINT PRIMARY KEY,
    question_id BIGINT NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    is_correct BOOLEAN NOT NULL
);

-- 8. Tags
CREATE TABLE tags (
    id BIGINT PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE
);

-- 9. Quiz Tags (Join Table)
CREATE TABLE quiz_tags (
    quiz_id BIGINT NOT NULL REFERENCES quizzes(id) ON DELETE CASCADE,
    tag_id BIGINT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (quiz_id, tag_id)
);

-- 10. User Answers (History) - Note: separate from developers in this context?
-- The previous schema used `users` table for both auth and this.
-- If we follow the "Pivot", `developers` manage API, but end-users solve quizzes?
-- The test suite registers `users` via `POST /users` (which might be distinct from `developers`).
-- But in the restored code `users` table was used for everything.
-- Let's assume for now `developers` are the ONLY user table, or re-introduce `users` if needed.
-- Wait, the test `test_b2b_managed_learning_flow` registers an end user via `POST /users`.
-- If I renamed `users` to `developers`, what happens to `POST /users`?
-- In the deleted migration checks (Step 2150), it renamed `users` to `developers` AND dropped `user_answers`.
-- But `models.rs` still has `SubmitAnswerRequest`. 
-- The user said "refined the code to use tid".
-- I will keep `user_answers` but link it to what?
-- If `POST /users` creates a "End User", where is that table?
-- Let's allow `user_answers` to store `user_email` or similar if no user table, OR keep `users` table distinct.
-- BUT, in `migrations/20250108000000_b2b_pivot.sql` (Step 2150), it said `ALTER TABLE users RENAME TO developers`.
-- And `POST /users` in `test_b2b_managed_learning_flow` seemed to use `user_email`.
-- Let's check `handlers.rs` for `create_user` or similar.
-- Actually, the best bet is to KEEP `users` table if the tests rely on it, OR assume `developers` replaces it.
-- But `test_b2b_managed_learning_flow` in `Step 2103` uses `POST /users` with `email` only clearly.
-- While `auth` uses `register` (username/password) for presumably developers.
-- I will add a `users` table for quiz-takers (end users) if needed, OR just make `user_answers` reference a string?
-- In `test_b2b_managed_learning_flow`, `user_email` is passed.
-- Let's make `user_answers` independent of a `users` table if `POST /users` just logs them or something.
-- OR define `users` table as:
CREATE TABLE users (
    id BIGINT PRIMARY KEY,
    email TEXT UNIQUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- And `user_answers` references `users(id)`.
-- Let's verify `handlers.rs` code later. For now, creating a schema that supports everything seen.

CREATE TABLE end_users (
    id BIGINT PRIMARY KEY, 
    email TEXT UNIQUE NOT NULL
);

CREATE TABLE user_answers (
    id BIGINT PRIMARY KEY,
    user_id BIGINT REFERENCES end_users(id) ON DELETE CASCADE, 
    -- OR if code expects string user_id, I'll check handlers. 
    -- But since we are moving to TSID, let's assume strict relations.
    quiz_id BIGINT NOT NULL REFERENCES quizzes(id) ON DELETE CASCADE,
    question_id BIGINT NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    option_id BIGINT NOT NULL REFERENCES question_options(id) ON DELETE CASCADE,
    is_correct BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- NOTE: If the code uses `user_email` directly in `user_answers` without a users table, I'll adjust. 
-- But typically a table is better.

