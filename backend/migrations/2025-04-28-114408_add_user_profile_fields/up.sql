-- Add user profile fields (pronouns, avatar_url, banner_url) to the users table

-- Add pronouns column (optional field)
ALTER TABLE users ADD COLUMN pronouns VARCHAR(50);

-- Add avatar_url column (optional field for profile image)
ALTER TABLE users ADD COLUMN avatar_url VARCHAR(255);

-- Add banner_url column (optional field for profile banner image)
ALTER TABLE users ADD COLUMN banner_url VARCHAR(255);

-- Add comments to explain the fields
COMMENT ON COLUMN users.pronouns IS 'Optional pronouns for the user (e.g., he/him, she/her, they/them)';
COMMENT ON COLUMN users.avatar_url IS 'URL to the user profile avatar image';
COMMENT ON COLUMN users.banner_url IS 'URL to the user profile banner image'; 