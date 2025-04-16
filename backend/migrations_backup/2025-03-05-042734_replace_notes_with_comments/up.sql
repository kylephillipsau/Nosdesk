-- Your SQL goes here

-- First, update attachments table to remove the dependency on notes
ALTER TABLE attachments
DROP COLUMN note_id;

-- Now we can safely drop the notes table
DROP TABLE IF EXISTS notes;

-- Create the new comments table
CREATE TABLE comments (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user_uuid VARCHAR(36) NOT NULL REFERENCES users(uuid),
    ticket_id INTEGER NOT NULL REFERENCES tickets(id)
);

-- Add comment_id to attachments table
ALTER TABLE attachments
ADD COLUMN comment_id INTEGER REFERENCES comments(id);
