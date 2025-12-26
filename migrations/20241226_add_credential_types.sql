-- Add credential_types table for user-configurable credential types
CREATE TABLE IF NOT EXISTS credential_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id),
    name VARCHAR(50) NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    icon VARCHAR(50) DEFAULT 'key',
    color VARCHAR(20) DEFAULT 'gray',
    is_system BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(team_id, name)
);

-- Insert default system types for each team (triggered by application on team creation)
-- For now, we'll add them via a function or manually

CREATE INDEX IF NOT EXISTS idx_credential_types_team_id ON credential_types(team_id);
CREATE INDEX IF NOT EXISTS idx_credential_types_name ON credential_types(team_id, name);
