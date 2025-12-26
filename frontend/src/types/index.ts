// User types
export interface User {
    id: string;
    email: string;
    name?: string;
    avatar_url?: string;
    role: 'admin' | 'developer' | 'viewer';
    team_id: string;
    created_at: string;
    updated_at?: string;
}

// Agent types
export interface Agent {
    id: string;
    team_id: string;
    name: string;
    description?: string;
    status: 'active' | 'inactive';
    usage_count: number;
    created_by: string;
    created_at: string;
    updated_at: string;
}

// Credential types
export interface Credential {
    id: string;
    agent_id: string;
    team_id: string;
    name: string;
    credential_type: string;
    description?: string;
    is_active: boolean;
    rotation_enabled: boolean;
    rotation_interval_days?: number;
    last_rotated?: string;
    next_rotation_due?: string;
    last_accessed?: string;
    created_at: string;
    updated_at: string;
}

// API Key types
export interface ApiKey {
    id: string;
    key?: string;
    key_prefix?: string;
    name: string;
    status: 'active' | 'revoked';
    last_used?: string;
    expires_at?: string;
    created_at: string;
}

// Ephemeral Token types
export interface EphemeralToken {
    token: string;
    expires_in: number;
    credential_type: string;
    credential_name: string;
    token_type: string;
}

// Response types
export interface ApiResponse<T> {
    data: T;
    message?: string;
    error?: string;
}

export interface PaginatedResponse<T> {
    data: T[];
    total: number;
    page: number;
    limit: number;
    pages: number;
}

// Form types
export interface LoginFormData {
    email: string;
    password: string;
    remember_me?: boolean;
}

export interface SignupFormData {
    email: string;
    password: string;
    team_name?: string;
}


export interface CreateAgentData {
    name: string;
    description?: string;
}

export interface CreateAgentResponse {
    agent: Agent;
    api_key: string;
    warning: string;
}

export interface UpdateAgentData {
    description?: string;
    status?: 'active' | 'inactive';
}

export interface CreateCredentialData {
    name: string;
    agent_id: string;
    credential_type: string;
    description?: string;
    secret: string;
    rotation_enabled?: boolean;
    rotation_interval_days?: number;
}

export interface UpdateCredentialData {
    description?: string;
    rotation_enabled?: boolean;
    rotation_interval_days?: number;
    secret?: string;
}

export interface CreateApiKeyData {
    name: string;
    expires_in_days?: number;
}
