-- This file should undo anything in `up.sql`

-- First, remove the comment_id column from attachments
ALTER TABLE attachments
DROP COLUMN comment_id;

-- Drop the comments table
DROP TABLE IF EXISTS comments;

-- Recreate the original notes table
CREATE TABLE notes (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    author VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ticket_id INTEGER REFERENCES tickets(id),
    is_comment BOOLEAN NOT NULL DEFAULT FALSE
);

-- Add note_id back to attachments
ALTER TABLE attachments
ADD COLUMN note_id INTEGER REFERENCES notes(id);
