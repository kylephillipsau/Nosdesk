-- This file should undo anything in `up.sql`

-- Revert the modified column back to DATE
ALTER TABLE tickets 
ALTER COLUMN modified TYPE DATE USING modified::DATE;

-- Update the default value back to CURRENT_DATE
ALTER TABLE tickets 
ALTER COLUMN modified SET DEFAULT CURRENT_DATE;
