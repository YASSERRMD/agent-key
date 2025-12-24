-- AgentKey Database Schema
-- Migration: 20241224000001_init_schema
-- Description: Initial schema with all core tables for agent credential management

-- Create custom types
CREATE TYPE credential_type AS ENUM ('api_key', 'oauth', 'db_connection', 'webhook', 'custom');
CREATE TYPE agent_status AS ENUM ('active', 'inactive', 'suspended');
CREATE TYPE user_role AS ENUM ('admin', 'developer', 'viewer');
CREATE TYPE plan_tier AS ENUM ('free', 'pro', 'enterprise');
CREATE TYPE action_type AS ENUM ('read', 'rotate', 'create', 'update', 'delete', 'revoke');
CREATE TYPE log_status AS ENUM ('success', 'failed');

-- ============================================================================
-- TEAMS TABLE (Multi-tenancy)
-- ============================================================================
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    owner_id UUID NOT NULL,
    plan plan_tier DEFAULT 'free' NOT NULL,
    max_agents INT DEFAULT 3 NOT NULL,
    max_credentials INT DEFAULT 10 NOT NULL,
    max_monthly_reads INT DEFAULT 1000 NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP WITH TIME ZONE
);

-- ============================================================================
-- USERS TABLE
-- ============================================================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    team_id UUID REFERENCES teams(id) ON DELETE CASCADE,
    role user_role DEFAULT 'developer' NOT NULL,
    is_active BOOLEAN DEFAULT true NOT NULL,
    last_login TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP WITH TIME ZONE,
    CONSTRAINT users_email_unique UNIQUE (email)
);

-- Add foreign key constraint after users table exists
ALTER TABLE teams ADD CONSTRAINT fk_teams_owner FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;

-- ============================================================================
-- AGENTS TABLE (AI Agents)
-- ============================================================================
CREATE TABLE agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status agent_status DEFAULT 'active' NOT NULL,
    framework VARCHAR(50),
    config JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_active TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    CONSTRAINT agents_team_name_unique UNIQUE (team_id, name)
);

-- ============================================================================
-- CREDENTIALS TABLE
-- ============================================================================
CREATE TABLE credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    credential_type credential_type NOT NULL,
    encrypted_value TEXT NOT NULL,
    encryption_key_version INT DEFAULT 1 NOT NULL,
    rotation_enabled BOOLEAN DEFAULT true NOT NULL,
    rotation_interval_minutes INT DEFAULT 60 NOT NULL,
    metadata JSONB,
    last_rotated TIMESTAMP WITH TIME ZONE,
    next_rotation TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) DEFAULT 'active' NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP WITH TIME ZONE,
    CONSTRAINT credentials_agent_name_unique UNIQUE (agent_id, name)
);

-- ============================================================================
-- EPHEMERAL TOKENS TABLE
-- ============================================================================
CREATE TABLE ephemeral_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    credential_id UUID NOT NULL REFERENCES credentials(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    token_prefix VARCHAR(10),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    usage_count INT DEFAULT 0 NOT NULL,
    max_usages INT,
    last_used TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    revoked_at TIMESTAMP WITH TIME ZONE,
    CONSTRAINT ephemeral_tokens_hash_unique UNIQUE (token_hash)
);

-- ============================================================================
-- CREDENTIAL ACCESS LOGS (Audit Trail)
-- ============================================================================
CREATE TABLE credential_access_logs (
    id BIGSERIAL PRIMARY KEY,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    credential_id UUID NOT NULL REFERENCES credentials(id) ON DELETE CASCADE,
    action action_type NOT NULL,
    status log_status NOT NULL,
    token_prefix VARCHAR(10),
    ip_address INET,
    user_agent VARCHAR(500),
    error_message TEXT,
    accessed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- ============================================================================
-- AUDIT EVENTS (Immutable)
-- ============================================================================
CREATE TABLE audit_events (
    id BIGSERIAL PRIMARY KEY,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    event_type VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50),
    resource_id UUID,
    old_values JSONB,
    new_values JSONB,
    change_description TEXT,
    ip_address INET,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- ============================================================================
-- API KEYS TABLE (SDK Authentication)
-- ============================================================================
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    key_hash VARCHAR(255) NOT NULL,
    key_prefix VARCHAR(10),
    name VARCHAR(255),
    last_used TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE,
    CONSTRAINT api_keys_hash_unique UNIQUE (key_hash)
);

-- ============================================================================
-- USAGE METRICS TABLE
-- ============================================================================
CREATE TABLE usage_metrics (
    id BIGSERIAL PRIMARY KEY,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    agent_id UUID REFERENCES agents(id) ON DELETE CASCADE,
    month DATE NOT NULL,
    credential_reads INT DEFAULT 0 NOT NULL,
    credential_rotations INT DEFAULT 0 NOT NULL,
    ephemeral_tokens_created INT DEFAULT 0 NOT NULL,
    api_calls INT DEFAULT 0 NOT NULL,
    storage_bytes BIGINT DEFAULT 0 NOT NULL,
    cost_cents INT DEFAULT 0 NOT NULL,
    CONSTRAINT usage_metrics_unique UNIQUE (team_id, agent_id, month)
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Teams indexes
CREATE INDEX idx_teams_owner ON teams(owner_id);
CREATE INDEX idx_teams_plan ON teams(plan);

-- Users indexes
CREATE INDEX idx_users_team ON users(team_id);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_active ON users(is_active) WHERE is_active = true;

-- Agents indexes
CREATE INDEX idx_agents_team ON agents(team_id);
CREATE INDEX idx_agents_status ON agents(status);
CREATE INDEX idx_agents_team_status ON agents(team_id, status);

-- Credentials indexes
CREATE INDEX idx_credentials_agent ON credentials(agent_id);
CREATE INDEX idx_credentials_status ON credentials(status);
CREATE INDEX idx_credentials_expires ON credentials(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_credentials_next_rotation ON credentials(next_rotation) WHERE rotation_enabled = true;

-- Ephemeral tokens indexes
CREATE INDEX idx_ephemeral_credential ON ephemeral_tokens(credential_id);
CREATE INDEX idx_ephemeral_expires ON ephemeral_tokens(expires_at);
CREATE INDEX idx_ephemeral_active ON ephemeral_tokens(expires_at) WHERE revoked_at IS NULL;

-- Access logs indexes
CREATE INDEX idx_access_logs_agent ON credential_access_logs(agent_id);
CREATE INDEX idx_access_logs_credential ON credential_access_logs(credential_id);
CREATE INDEX idx_access_logs_agent_time ON credential_access_logs(agent_id, accessed_at DESC);
CREATE INDEX idx_access_logs_time ON credential_access_logs(accessed_at DESC);

-- Audit events indexes
CREATE INDEX idx_audit_events_team ON audit_events(team_id);
CREATE INDEX idx_audit_events_team_time ON audit_events(team_id, created_at DESC);
CREATE INDEX idx_audit_events_resource ON audit_events(resource_type, resource_id);
CREATE INDEX idx_audit_events_user ON audit_events(user_id) WHERE user_id IS NOT NULL;

-- API keys indexes
CREATE INDEX idx_api_keys_agent ON api_keys(agent_id);
CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix);

-- Usage metrics indexes
CREATE INDEX idx_usage_metrics_team_month ON usage_metrics(team_id, month DESC);
CREATE INDEX idx_usage_metrics_agent ON usage_metrics(agent_id) WHERE agent_id IS NOT NULL;

-- ============================================================================
-- TRIGGERS
-- ============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply updated_at triggers
CREATE TRIGGER update_teams_updated_at
    BEFORE UPDATE ON teams
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_agents_updated_at
    BEFORE UPDATE ON agents
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_credentials_updated_at
    BEFORE UPDATE ON credentials
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
