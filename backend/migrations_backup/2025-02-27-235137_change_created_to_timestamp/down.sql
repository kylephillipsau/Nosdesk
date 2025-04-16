-- This file should undo anything in `up.sql`
ALTER TABLE tickets
ALTER COLUMN created TYPE DATE USING (created::DATE);