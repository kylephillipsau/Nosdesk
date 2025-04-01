-- Your SQL goes here

-- Add parent_id column to documentation_pages table
ALTER TABLE documentation_pages
ADD COLUMN parent_id INTEGER REFERENCES documentation_pages(id);

-- Create an index on parent_id for faster queries
CREATE INDEX idx_documentation_pages_parent_id ON documentation_pages(parent_id);

-- We don't need the documentation_page_relationships table anymore since we're using a parent_id field
-- But we'll keep it for now and migrate the data
-- We can drop it in a future migration if needed
