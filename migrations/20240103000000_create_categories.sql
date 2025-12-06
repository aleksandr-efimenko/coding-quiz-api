CREATE TABLE categories (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

ALTER TABLE quizzes ADD COLUMN category_id UUID REFERENCES categories(id);
