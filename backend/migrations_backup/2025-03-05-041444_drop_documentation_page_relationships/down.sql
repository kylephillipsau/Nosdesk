-- This file should undo anything in `up.sql`

-- Recreate the documentation_page_relationships table if the migration is reverted
CREATE TABLE documentation_page_relationships (
    parent_id INTEGER NOT NULL REFERENCES documentation_pages(id),
    child_id INTEGER NOT NULL REFERENCES documentation_pages(id),
    display_order INTEGER NOT NULL,
    PRIMARY KEY (parent_id, child_id)
);
