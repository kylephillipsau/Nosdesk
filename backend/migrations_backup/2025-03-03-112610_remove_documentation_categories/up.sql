-- Your SQL goes here

-- First remove the foreign key constraint from documentation_pages
ALTER TABLE documentation_pages DROP CONSTRAINT IF EXISTS documentation_pages_category_id_fkey;

-- Drop the category_id column from documentation_pages
ALTER TABLE documentation_pages DROP COLUMN category_id;

-- Drop the documentation_categories table
DROP TABLE IF EXISTS documentation_categories;
