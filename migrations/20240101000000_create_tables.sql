CREATE TABLE quizzes (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL
);

CREATE TABLE questions (
    id UUID PRIMARY KEY,
    quiz_id UUID NOT NULL REFERENCES quizzes(id),
    text TEXT NOT NULL
);

CREATE TABLE question_options (
    id UUID PRIMARY KEY,
    question_id UUID NOT NULL REFERENCES questions(id),
    text TEXT NOT NULL,
    is_correct BOOLEAN NOT NULL
);
