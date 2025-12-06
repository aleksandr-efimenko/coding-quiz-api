-- Add unique constraint to avoid duplicate questions
ALTER TABLE questions ADD CONSTRAINT questions_text_key UNIQUE (text);
