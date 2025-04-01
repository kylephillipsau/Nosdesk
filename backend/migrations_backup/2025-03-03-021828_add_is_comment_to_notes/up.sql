-- Your SQL goes here

-- Add is_comment column to notes table
ALTER TABLE notes ADD COLUMN is_comment BOOLEAN NOT NULL DEFAULT TRUE;

-- Create an index for better performance when filtering
CREATE INDEX idx_notes_is_comment ON notes(is_comment);

-- Update existing notes to be comments
UPDATE notes SET is_comment = TRUE;
