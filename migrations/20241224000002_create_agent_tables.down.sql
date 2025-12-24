-- Drop triggers
DROP TRIGGER IF EXISTS update_agent_quotas_updated_at ON agent_quotas;
DROP TRIGGER IF EXISTS update_agents_updated_at ON agents;

-- Drop indexes (implicitly dropped with tables, but good practice to be explicit if needed, though usually not)
-- SQL handles index drops with table drops

-- Drop tables in reverse order of dependencies
DROP TABLE IF EXISTS agent_usage;
DROP TABLE IF EXISTS agent_quotas;
DROP TABLE IF EXISTS agent_api_keys;
DROP TABLE IF EXISTS agents;
