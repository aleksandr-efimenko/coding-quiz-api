-- Add user_answers table
CREATE TABLE user_answers (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    quiz_id UUID NOT NULL REFERENCES quizzes(id) ON DELETE CASCADE,
    question_id UUID NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    option_id UUID NOT NULL REFERENCES question_options(id) ON DELETE CASCADE,
    is_correct BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for faster history lookup
CREATE INDEX idx_user_answers_user_id ON user_answers(user_id);
-- Index for analytics (optional but good)
CREATE INDEX idx_user_answers_quiz_id ON user_answers(quiz_id);
