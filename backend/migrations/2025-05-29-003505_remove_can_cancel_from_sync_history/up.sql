-- Your SQL goes here

-- Remove the can_cancel column from sync_history table
ALTER TABLE sync_history DROP COLUMN IF EXISTS can_cancel;
