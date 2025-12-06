-- Enable cascade delete for questions
ALTER TABLE questions DROP CONSTRAINT questions_quiz_id_fkey;
ALTER TABLE questions ADD CONSTRAINT questions_quiz_id_fkey 
    FOREIGN KEY (quiz_id) REFERENCES quizzes(id) ON DELETE CASCADE;

-- Enable cascade delete for question_options
ALTER TABLE question_options DROP CONSTRAINT question_options_question_id_fkey;
ALTER TABLE question_options ADD CONSTRAINT question_options_question_id_fkey 
    FOREIGN KEY (question_id) REFERENCES questions(id) ON DELETE CASCADE;
