-- Database initialization script for Nosdesk
-- This script ensures the database and user are properly configured

-- Create database if it doesn't exist (PostgreSQL)
-- Note: In Docker, the database is already created via POSTGRES_DB environment variable

-- Ensure the user has the necessary permissions
GRANT ALL PRIVILEGES ON DATABASE helpdesk TO nosdesk;

-- Create any additional schema or initial data here if needed
-- The actual table schema will be created by Diesel migrations 