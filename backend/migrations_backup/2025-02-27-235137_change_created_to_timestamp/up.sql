-- Your SQL goes here
ALTER TABLE tickets
ALTER COLUMN created TYPE TIMESTAMP USING (created::TIMESTAMP);