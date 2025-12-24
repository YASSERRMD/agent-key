-- credentials table
CREATE TABLE credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id),
    team_id UUID NOT NULL REFERENCES teams(id),
    name VARCHAR(255) NOT NULL,
    credential_type VARCHAR(50) NOT NULL,  -- api_key, password, token, ssh_key, oauth, custom
    description TEXT,
    encrypted_value BYTEA NOT NULL,  -- [nonce || ciphertext || tag]
    is_active BOOLEAN DEFAULT true,
    last_accessed TIMESTAMP WITH TIME ZONE,
    rotation_enabled BOOLEAN DEFAULT false,
    rotation_interval_days INT,  -- null if rotation disabled
    last_rotated TIMESTAMP WITH TIME ZONE,
    next_rotation_due TIMESTAMP WITH TIME ZONE,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(agent_id, name)  -- unique credential name per agent
);

-- credential_versions table (for rotation tracking)
CREATE TABLE credential_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    credential_id UUID NOT NULL REFERENCES credentials(id),
    version INT NOT NULL,
    encrypted_value BYTEA NOT NULL,  -- [nonce || ciphertext || tag]
    status VARCHAR(50) DEFAULT 'active',  -- active, superseded, archived
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expired_at TIMESTAMP WITH TIME ZONE
);

-- credential_access_log table (audit trail)
CREATE TABLE credential_access_log (
    id BIGSERIAL PRIMARY KEY,
    credential_id UUID NOT NULL REFERENCES credentials(id),
    agent_id UUID NOT NULL REFERENCES agents(id),
    team_id UUID NOT NULL REFERENCES teams(id),
    access_type VARCHAR(50) NOT NULL,  -- read, decrypt, update, delete
    status VARCHAR(50) NOT NULL,  -- success, failed
    reason TEXT,
    ip_address INET,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- encryption_key_material table (for key rotation in future)
CREATE TABLE encryption_key_material (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_version INT NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    rotated_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) DEFAULT 'active',  -- active, rotated, archived
    UNIQUE(key_version)
);

-- Create indexes for performance
CREATE INDEX idx_credentials_agent_id ON credentials(agent_id);
CREATE INDEX idx_credentials_team_id ON credentials(team_id);
CREATE INDEX idx_credentials_active ON credentials(is_active) WHERE deleted_at IS NULL;
CREATE INDEX idx_credential_versions_credential_id ON credential_versions(credential_id);
CREATE INDEX idx_credential_access_log_credential_id ON credential_access_log(credential_id);
CREATE INDEX idx_credential_access_log_agent_id ON credential_access_log(agent_id);
CREATE INDEX idx_credential_access_log_timestamp ON credential_access_log(timestamp);
CREATE INDEX idx_encryption_key_material_version ON encryption_key_material(key_version);
