-- This file should undo anything in `up.sql`

-- Add back the can_cancel column to sync_history table
ALTER TABLE sync_history ADD COLUMN can_cancel BOOLEAN NOT NULL DEFAULT false;
