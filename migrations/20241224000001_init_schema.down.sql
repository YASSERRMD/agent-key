-- Rollback Migration: 20241224000001_init_schema
-- Description: Drop all tables and types created in the up migration

-- Drop triggers first
DROP TRIGGER IF EXISTS update_credentials_updated_at ON credentials;
DROP TRIGGER IF EXISTS update_agents_updated_at ON agents;
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
DROP TRIGGER IF EXISTS update_teams_updated_at ON teams;

-- Drop function
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop tables in reverse order of creation (respecting foreign keys)
DROP TABLE IF EXISTS usage_metrics;
DROP TABLE IF EXISTS api_keys;
DROP TABLE IF EXISTS audit_events;
DROP TABLE IF EXISTS credential_access_logs;
DROP TABLE IF EXISTS ephemeral_tokens;
DROP TABLE IF EXISTS credentials;
DROP TABLE IF EXISTS agents;

-- Drop foreign key constraint before dropping users
ALTER TABLE IF EXISTS teams DROP CONSTRAINT IF EXISTS fk_teams_owner;

DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS teams;

-- Drop custom types
DROP TYPE IF EXISTS log_status;
DROP TYPE IF EXISTS action_type;
DROP TYPE IF EXISTS plan_tier;
DROP TYPE IF EXISTS user_role;
DROP TYPE IF EXISTS agent_status;
DROP TYPE IF EXISTS credential_type;
