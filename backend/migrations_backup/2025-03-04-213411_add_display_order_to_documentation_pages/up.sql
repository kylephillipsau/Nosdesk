-- Your SQL goes here

-- Add display_order column to documentation_pages table
ALTER TABLE documentation_pages ADD COLUMN display_order INTEGER DEFAULT 0;

-- Create an index for better performance
CREATE INDEX idx_documentation_pages_display_order ON documentation_pages(display_order);

-- Update existing pages with sequential display_order values
-- First, set display_order for top-level pages (parent_id IS NULL)
WITH ordered_pages AS (
  SELECT id, ROW_NUMBER() OVER (ORDER BY title) as row_num
  FROM documentation_pages
  WHERE parent_id IS NULL
)
UPDATE documentation_pages
SET display_order = ordered_pages.row_num
FROM ordered_pages
WHERE documentation_pages.id = ordered_pages.id;

-- Then, set display_order for child pages (grouped by parent_id)
WITH ordered_children AS (
  SELECT id, parent_id, ROW_NUMBER() OVER (PARTITION BY parent_id ORDER BY title) as row_num
  FROM documentation_pages
  WHERE parent_id IS NOT NULL
)
UPDATE documentation_pages
SET display_order = ordered_children.row_num
FROM ordered_children
WHERE documentation_pages.id = ordered_children.id;
