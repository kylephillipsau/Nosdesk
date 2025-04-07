-- This file should undo anything in `up.sql`

-- Drop indices
DROP INDEX IF EXISTS idx_auth_providers_type;
DROP INDEX IF EXISTS idx_auth_provider_configs_provider_id;

-- Drop tables in reverse order (respecting foreign keys)
DROP TABLE IF EXISTS auth_provider_configs;
DROP TABLE IF EXISTS auth_providers;
