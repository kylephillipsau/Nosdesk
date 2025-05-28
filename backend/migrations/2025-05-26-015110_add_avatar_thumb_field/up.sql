-- Your SQL goes here

-- Add avatar_thumb field to users table for 48x48 thumbnails
ALTER TABLE users ADD COLUMN avatar_thumb VARCHAR(255);

-- Update existing users to populate avatar_thumb from avatar_url if it contains 120x120
-- This will help with the transition by copying existing 120x120 URLs and transforming them to 48x48
UPDATE users 
SET avatar_thumb = REPLACE(avatar_url, '120x120', '48x48')
WHERE avatar_url IS NOT NULL 
  AND avatar_url LIKE '%120x120%';
