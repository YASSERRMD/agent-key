-- Add team_id to credentials (idempotent)
ALTER TABLE credentials ADD COLUMN IF NOT EXISTS team_id UUID REFERENCES teams(id) ON DELETE CASCADE;

-- Add description to credentials (idempotent)
ALTER TABLE credentials ADD COLUMN IF NOT EXISTS description TEXT;

-- Add last_accessed to credentials (idempotent)
ALTER TABLE credentials ADD COLUMN IF NOT EXISTS last_accessed TIMESTAMP WITH TIME ZONE;

-- Create credential_versions table (idempotent-ish check)
CREATE TABLE IF NOT EXISTS credential_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    credential_id UUID NOT NULL REFERENCES credentials(id) ON DELETE CASCADE,
    version INT NOT NULL,
    encrypted_value BYTEA NOT NULL,
    status VARCHAR(50) DEFAULT 'active' NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expired_at TIMESTAMP WITH TIME ZONE
);

-- Add 'generic' to credential_type enum (if not exists)
ALTER TYPE credential_type ADD VALUE IF NOT EXISTS 'generic';

-- Backfill data
UPDATE credentials c
SET team_id = a.team_id
FROM agents a
WHERE c.agent_id = a.id AND c.team_id IS NULL;

-- Make it NOT NULL
ALTER TABLE credentials ALTER COLUMN team_id SET NOT NULL;
