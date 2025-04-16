-- This file should undo anything in `up.sql`

-- Recreate the documentation_categories table
CREATE TABLE IF NOT EXISTS documentation_categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    icon VARCHAR(50),
    parent_id INTEGER REFERENCES documentation_categories(id),
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

-- Add the category_id column back to documentation_pages
ALTER TABLE documentation_pages ADD COLUMN category_id INTEGER REFERENCES documentation_categories(id);
