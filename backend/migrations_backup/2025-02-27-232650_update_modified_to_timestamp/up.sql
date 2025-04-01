-- Your SQL goes here

-- Change the modified column from DATE to TIMESTAMP
ALTER TABLE tickets 
ALTER COLUMN modified TYPE TIMESTAMP USING modified::TIMESTAMP;

-- Update the default value to use CURRENT_TIMESTAMP
ALTER TABLE tickets 
ALTER COLUMN modified SET DEFAULT CURRENT_TIMESTAMP;
