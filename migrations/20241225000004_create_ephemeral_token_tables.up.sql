-- Ephemeral token tables for Phase 4
-- Created: 2024-12-25

-- ephemeral_tokens: Track generated tokens
CREATE TABLE ephemeral_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jti VARCHAR(255) NOT NULL UNIQUE,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    credential_id UUID NOT NULL REFERENCES credentials(id) ON DELETE CASCADE,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    token_signature VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    revoked_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- token_usage_log: Audit trail for token actions
CREATE TABLE token_usage_log (
    id BIGSERIAL PRIMARY KEY,
    jti VARCHAR(255) NOT NULL,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,
    ip_address INET,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- sdk_sessions: Track SDK client sessions
CREATE TABLE sdk_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    sdk_version VARCHAR(20) NOT NULL,
    user_agent TEXT,
    last_activity TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Performance indexes
CREATE INDEX idx_ephemeral_tokens_agent_id ON ephemeral_tokens(agent_id);
CREATE INDEX idx_ephemeral_tokens_credential_id ON ephemeral_tokens(credential_id);
CREATE INDEX idx_ephemeral_tokens_status ON ephemeral_tokens(status) WHERE status = 'active';
CREATE INDEX idx_ephemeral_tokens_expires_at ON ephemeral_tokens(expires_at);
CREATE INDEX idx_token_usage_log_jti ON token_usage_log(jti);
CREATE INDEX idx_token_usage_log_agent_id ON token_usage_log(agent_id);
CREATE INDEX idx_token_usage_log_timestamp ON token_usage_log(timestamp);
CREATE INDEX idx_sdk_sessions_agent_id ON sdk_sessions(agent_id);
